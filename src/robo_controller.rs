use std_robo::*;
use std_robo::UserCmd::*;
use std_robo::Resp::*;

use cgmath::Vector2;

use serde_json;

use std::thread;
use std::thread::JoinHandle;
use std::io;
use std::io::{BufReader, BufRead, Write};
use std::sync::mpsc::{Sender, Receiver, channel};

pub struct RoboHook {
    command_sender: Sender<UserCmd>,
    response_receiver: Receiver<Resp>,
}

pub struct RemoteRobo {
    hook: RoboHook,
    event_receiver: Receiver<Event>,

    reader_handle: JoinHandle<()>,
    writer_handle: JoinHandle<()>,
}

impl RemoteRobo {
    pub fn new() -> Self {
        let (cmd_out, cmd_in) = channel();
        let (resp_out, resp_in) = channel();
        let (ev_out, ev_in) = channel();

        let hook = RoboHook {
            command_sender: cmd_out,
            response_receiver: resp_in,
        };

        let reader_handle = thread::spawn(move|| {
            let stdin_1 = io::stdin();
            let stdin = BufReader::new(stdin_1.lock());

            for line in stdin.lines() {
                let line = line.unwrap();
                let output = serde_json::from_str(&line).unwrap();

                match output {
                    Output::Event(e) => ev_out.send(e).unwrap(),
                    Output::Resp(r) => resp_out.send(r).unwrap(),
                }
            }
        });

        let writer_handle = thread::spawn(move|| {
            let stdout_1 = io::stdout();
            let mut stdout = stdout_1.lock();

            for cmd in cmd_in {
                let s = serde_json::to_string(&cmd).unwrap();
                writeln!(stdout, "{}", s).unwrap();
            }
        });

        RemoteRobo {
            hook: hook,
            event_receiver: ev_in,
            reader_handle: reader_handle,
            writer_handle: writer_handle,
        }
    }

    pub fn run_controller<T: RoboController>(&self, ctl: &mut T) {
        for ev in &self.event_receiver {
            ctl.handle_event(&self.hook, ev);
        }
    }
}

impl RoboHook {
    pub fn get_pos(&self) -> Vector2<f64> {
        self.command_sender.send(GetPos).unwrap();

        match self.response_receiver.recv().unwrap() {
            PosIs(SerVector2(pos)) => pos,
            _ => unreachable!(),
        }
    }

    pub fn get_state(&self) -> StdRobo {
        self.command_sender.send(GetState).unwrap();

        match self.response_receiver.recv().unwrap() {
            StateIs(state) => state,
            _ => unreachable!(),
        }
    }

    pub fn log<S: Into<String>>(&self, msg: S) {
        self.command_sender.send(Log(msg.into())).unwrap();
    }

    pub fn set_vel(&self, new_vel: Vector2<f64>) {
        self.command_sender.send(SetVel(SerVector2(new_vel))).unwrap();
    }
}

pub trait RoboController {
    fn handle_event(&mut self, hook: &RoboHook, event: Event);
}
