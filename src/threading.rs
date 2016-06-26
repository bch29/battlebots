use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};

use std::panic::{UnwindSafe, catch_unwind};
use std::any::Any;

/// A thread coordinator, which simplifies the coordination of many threads with
/// the same return values.
pub struct Coordinator<T> where T: Send + 'static {
    receiver: Receiver<Response<T>>,
    sender: Sender<Response<T>>,
    handles: Vec<thread::JoinHandle<()>>,
}

impl<T> Coordinator<T> where T: Send + 'static {
    /// Creates a new, initially empty, thread coordinator.
    pub fn new() -> Self {
        let (tx, rx) = channel();

        Coordinator {
            receiver: rx,
            sender: tx,
            handles: Vec::new(),
        }
    }

    /// Adds a task to the thread coordinator which runs the provided closure,
    /// starting it immediately.
    pub fn spawn<F>(&mut self, f: F)
        where F: FnOnce() -> T, F: Send + 'static + UnwindSafe {

        let id = self.handles.len();
        let sender = self.sender.clone();

        let handle = thread::spawn(move|| {
            let res = catch_unwind(f);

            sender.send(Response {
                id: id,
                response: res,
            }).expect("`Coordinator` object hung up");
        });

        self.handles.push(handle);
    }

    /// Waits until any of the running threads either completes or panics.
    /// Returns either the result or panic value, then detaches all other
    /// running threads (but does _not_ stop them).
    pub fn wait_first(self) -> thread::Result<T> {
        self.receiver.recv().expect("`Coordinator` object hung up").response
    }

    /// Waits until all running threads have either completed or panicked.
    /// Results (or panic values) are returned in the order the threads were
    /// spawned.
    pub fn wait_all(self) -> Vec<thread::Result<T>> {
        let mut responses: Vec<_> = self.handles.iter().map(|_| {
            Err(Box::new("Thread not responded".to_owned()) as Box<Any + 'static + Send>)
        }).collect();

        for resp in self.receiver {
            responses[resp.id] = resp.response;
        }

        responses
    }
}

struct Response<T> {
    id: usize,
    response: thread::Result<T>,
}
