use config::Config;
use world::{TickLock};
use ctl::RoboCtl;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver, channel};

use std::thread;

use std::time::Instant;

#[derive(Debug)]
struct State<Ctl: RoboCtl> {
    ctl: Ctl,
    msg_in: Receiver<Message>,
    msg_out: Sender<Message>,
}

/// An asynchronous robot, whose behaviour is determined by the `Ctl` type.
pub struct Robo<Ctl: RoboCtl> {
    config: Config,
    state: Mutex<State<Ctl>>,
}

#[derive(Debug)]
pub enum Error<Ctl: RoboCtl> {
    StatePoisoned,
    WorldPoisoned,
    Ctl(Ctl::Error),
}

#[derive(Debug)]
pub enum StateError {
    Poisoned
}

#[derive(Debug)]
pub enum Message {
    Shutdown,
}

impl<Ctl: RoboCtl> Robo<Ctl> {
    /// Creates a new robot in the given world. Requires taking a write lock on
    /// the world and thus may fail if the world's lock in poisoned.
    pub fn new(config: Config, ctl: Ctl) -> Self {
        let (msg_out, msg_in) = channel();

        Robo {
            config: config,
            state: Mutex::new(State { ctl: ctl, msg_in: msg_in, msg_out: msg_out }),
        }
    }

    /// Gets a copy of the `Sender` that allows asynchronous communication with
    /// a running robot. Requires taking a lock on the underlying state and thus
    /// may fail if the lock is poisoned.
    pub fn msg_sender(&self) -> Result<Sender<Message>, StateError> {
        let state = try!(self.state.lock().map_err(|_| StateError::Poisoned));
        Ok(state.msg_out.clone())
    }

    /// Synchronously runs the robot. Should only be called once (from any
    /// thread) and should usually be run in a new thread. See `rob::run_async`.
    pub fn run(&self, tick_lock: &TickLock) -> Result<(), Error<Ctl>> {
        use self::Message::*;

        // Initialise in a block to make sure to drop the lock on the state when
        // done.
        {
            // Get a lock on the state
            let mut state = try!(self.state.lock().map_err(|_| Error::StatePoisoned));

            try!(state.ctl.init().map_err(Error::Ctl));
        }

        let mut prev_time = Instant::now();

        loop {
            // Wait until we are allowed to tick
            let _tick_guard = tick_lock.take();

            // Get a lock on the state
            let mut state = try!(self.state.lock().map_err(|_| Error::StatePoisoned));

            // Handle any waiting messages
            while let Ok(msg) = state.msg_in.try_recv() {
                match msg {
                    Shutdown => return Ok(()),
                }
            }

            // Tell the `Ctl` to tick
            let now = Instant::now();
            try!(state.ctl.tick(now.duration_since(prev_time)).map_err(Error::Ctl));
            prev_time = now;
        }
    }

    /// Do something with the underlying `Ctl` object.
    pub fn with_ctl<F, R>(&self, f: F) -> Result<R, StateError>
        where F: FnOnce(&Ctl) -> R {

        let state = try!(self.state.lock().map_err(|_| StateError::Poisoned));

        Ok(f(&state.ctl))
    }

    /// Asynchronously runs the robot in a new thread.
    ///
    /// ```
    /// use std::thread;
    /// use std::sync::Arc;
    ///
    /// let robo = Arc::new(/* A robot */);
    ///
    /// run_async(robo.clone());
    ///
    /// // do other things
    /// ```
    pub fn run_async(robo: Arc<Robo<Ctl>>, tick_lock: Arc<TickLock>) -> thread::JoinHandle<Result<(), Error<Ctl>>>
        where Ctl: Send + 'static {

        thread::spawn(move|| {
            robo.run(&*tick_lock)
        })
    }
}
