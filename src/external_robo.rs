use robo::*;
use std_robo::*;

use std::thread;
use std::io;
use std::sync::mpsc::{Sender, SendError};
use std::process::{Child, Command, Stdio};
use std::io::{Write, BufReader, BufRead};

use serde_json;

pub struct ExternalRobo {
    input_thread: thread::JoinHandle<()>,
    output_thread: thread::JoinHandle<()>,
    child_process: Child,
    command_sender: Sender<Cmd>,
    stopper: RoboStopper,
}

impl ExternalRobo {
    /// Creates a new `ExternalRobo` from an initial state and the `Command`
    /// which starts the controlling external process. Returns an error if the
    /// child process cannot be spawned.
    pub fn new(initial_state: StdRobo, mut external_command: Command) -> io::Result<ExternalRobo> {
        let robo = Robo::new(initial_state);

        let mut child = try!(external_command
                             .stdin(Stdio::piped())
                             .stdout(Stdio::piped())
                             .spawn());

        // Take ownership of the child's stdin and stdout.
        let child_out = child.stdout.expect("Child process standard output not available on this platform.");
        child.stdout = None;
        let mut child_in = child.stdin.expect("Child process standard input not available on this platform.");
        child.stdin = None;

        // We need two copies of the input sender, for the output thread and for
        // the `ExternalRobo` itself.
        let input_sender = robo.input_sender();
        let input_sender_2 = input_sender.clone();

        let stopper = robo.stopper();

        // Thread that relays output from the robo into the child's stdin.
        let input_thread = thread::spawn(move|| {
            while let Ok(resp) = robo.recv_output() {
                let s = serde_json::to_string(&resp).unwrap();
                if let Err(_) = writeln!(child_in, "{}", s) {
                    break;
                }

                if let Err(_) = child_in.flush() {
                    break;
                }
            }
        });

        // Thread that relays output from the child's stdout into the robo's
        // input.
        let output_thread = thread::spawn(move|| {
            for t_line in BufReader::new(child_out).lines() {
                if let Ok(line) = t_line {
                    if line.is_empty() {
                        break;
                    }

                    if let Ok(cmd) = serde_json::from_str(line.as_str()) {
                        input_sender.send(Cmd::UserCmd(cmd)).unwrap();
                    }
                }
            }
        });

        Ok(ExternalRobo {
            input_thread: input_thread,
            output_thread: output_thread,
            child_process: child,
            command_sender: input_sender_2,
            stopper: stopper,
        })
    }

    /// Sends a command to the robot's internal thread (not to the external process).
    pub fn send_cmd(&self, cmd: Cmd) -> Result<(), SendError<Cmd>> {
        self.command_sender.send(cmd)
    }

    /// Blocks while waiting for the robot's external process to finish.
    pub fn wait(mut self) {
        let _ = self.child_process.wait();
        let _ = self.output_thread.join();
        self.stopper.stop();
        let _ = self.input_thread.join();
    }
}
