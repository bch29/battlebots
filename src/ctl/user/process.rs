use rpc::*;

use std::sync::Arc;
use std::io;
use std::io::{BufRead, Write};

use serde_json;
use serde_json::error::Error as JsonError;
use crossbeam::sync::MsQueue;

pub struct Process<R, W> {
    output_writer: W,
    input_reader: R,
    relay: Arc<Relay>,
}

#[derive(Debug)]
pub enum ProcessError {
    Serialization(JsonError),
    Deserialization(JsonError),
    Writing(io::Error),
    Reading(io::Error),
}

pub struct Relay {
    msg_queue: MsQueue<(BotState, Message)>,
    resp_queue: MsQueue<Vec<Response>>,
}

impl<R, W> Process<R, W> {
    pub fn new(output_writer: W, input_reader: R) -> Self {
        Process {
            output_writer: output_writer,
            input_reader: input_reader,
            relay: Arc::new(Relay::new()),
        }
    }

    pub fn relay(&self) -> Arc<Relay> {
        self.relay.clone()
    }
}

impl<R, W> Process<R, W>
    where R: BufRead,
          W: Write
{
    pub fn run(mut self) -> Result<(), ProcessError> {
        loop {
            // Relay a waiting message from the queue to the child process
            let msg = self.relay.recv_msg();
            let ser = try!(serde_json::to_string(&msg).map_err(ProcessError::Serialization));
            try!(writeln!(self.output_writer, "{}", ser).map_err(ProcessError::Writing));

            // Receive a list of responses from the child process
            let mut next_line = String::new();
            try!(self.input_reader.read_line(&mut next_line).map_err(ProcessError::Reading));
            let resps = try!(serde_json::from_str(next_line.as_str())
                .map_err(ProcessError::Deserialization));
            self.relay.send_resps(resps);
        }
    }
}

impl Relay {
    pub fn new() -> Self {
        Relay {
            msg_queue: MsQueue::new(),
            resp_queue: MsQueue::new(),
        }
    }

    pub fn send_msg(&self, msg: (BotState, Message)) {
        self.msg_queue.push(msg);
    }

    pub fn try_recv_resps(&self) -> Option<Vec<Response>> {
        self.resp_queue.try_pop()
    }

    fn send_resps(&self, resp: Vec<Response>) {
        self.resp_queue.push(resp);
    }

    fn recv_msg(&self) -> (BotState, Message) {
        self.msg_queue.pop()
    }
}

impl Default for Relay {
    fn default() -> Self {
        Relay::new()
    }
}
