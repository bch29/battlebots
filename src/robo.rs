use std::sync::mpsc::{channel, Sender, Receiver, SendError, RecvError, TryRecvError};
use std::marker::PhantomData;
use std::thread;

/// A controller for an asynchronous robot, whose behaviour is determined by the
/// `Ctl` type.
pub struct Robo<Ctl: AsyncRobo> {
    _marker: PhantomData<Ctl>,
    input_out: Sender<Ctl::Input>,
    output_in: Receiver<Ctl::Output>,
    stopper: RoboStopper,
    join_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone)]
pub struct RoboStopper {
    stop_out: Sender<()>,
}

impl RoboStopper {
    /// Asynchronously stops the robot's execution as soon as currently buffered
    /// input messages have finished being handled. After one call to `stop()`,
    /// subsequent calls have no effect.
    pub fn stop(&self) {
        let _ = self.stop_out.send(());
    }
}

pub trait AsyncRobo: Send + 'static {
    /// The type of input messages that this robot can receive.
    type Input: Send;

    /// The type of output messages that this robot can send.
    type Output: Send;

    /// Handle a single input message, and optionally return a response to be
    /// sent to the owner of the controlling `Robo`.
    fn handle_input(&mut self, input: Self::Input) -> Option<Self::Output>;
}

impl<Ctl: AsyncRobo> Robo<Ctl> {
    /// Create a Robo with the given controller and start it in its own thread.
    pub fn new(ctl: Ctl) -> Self {
        let (input_out, input_in) = channel();
        let (output_out, output_in) = channel();
        let (stop_out, stop_in) = channel();

        let stopper = RoboStopper { stop_out: stop_out };

        let join_handle = thread::spawn(move|| {
            let mut s = ctl;

            loop {
                select! {
                    _ = stop_in.recv() => break,
                    input = input_in.recv() =>
                        if let Ok(input) = input {
                            match s.handle_input(input) {
                                Some(output) =>
                                    if let Err(_) = output_out.send(output) {
                                        break
                                    },
                                None => (),
                            }
                        } else { break }
                }
            }
        });

        Robo {
            _marker: PhantomData,
            input_out: input_out,
            output_in: output_in,
            stopper: stopper,
            join_handle: join_handle,
        }
    }

    /// Gets a copy of the sender that can be used to send inputs to the robot
    /// without having access to the `Robo` itself.
    pub fn input_sender(&self) -> Sender<Ctl::Input> {
        self.input_out.clone()
    }

    /// Sends an input message to the robot, returning an error if the robot has
    /// already stopped. The input is not guaranteed to be received if `stop()`
    /// is called following this function.
    pub fn send_input(&self, input: Ctl::Input) -> Result<(), SendError<Ctl::Input>> {
        self.input_out.send(input)
    }

    /// Asynchronously stops the robot's execution as soon as currently buffered
    /// input messages have finished being handled. After one call to `stop()`,
    /// subsequent calls have no effect.
    pub fn stop(&self) {
        self.stopper.stop();
    }

    /// Get an object that can be used to asynchronously stop the robot's
    /// execution as soon as currently buffered input messages have finished
    /// being handled.
    pub fn stopper(&self) -> RoboStopper {
        self.stopper.clone()
    }

    /// Stops the robot's execution as soon as the currently buffered input
    /// messages have finished being handled, and blocks until it has actually
    /// stopped. Consumes the robot.
    pub fn stop_wait(self) {
        self.stop();
        let _ = self.join_handle.join();
    }

    /// Blocks on an output message from the robot's asynchronous thread,
    /// returning one when it arrives.
    pub fn recv_output(&self) -> Result<Ctl::Output, RecvError> {
        self.output_in.recv()
    }

    /// Checks if there is a waiting output message from the robot's
    /// asynchronous thread, and returns it if so.
    pub fn try_recv_output(&self) -> Result<Ctl::Output, TryRecvError> {
        self.output_in.try_recv()
    }
}
