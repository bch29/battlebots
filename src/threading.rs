use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};

use std::panic::{UnwindSafe, catch_unwind};

/// A thread coordinator, which simplifies the coordination of many threads with
/// the same return values.
pub struct Coordinator<T>
    where T: Send + 'static
{
    receiver: Receiver<Response<T>>,
    sender: Sender<Response<T>>,

    count: usize,
    count_active: usize,
}

impl<T> Coordinator<T>
    where T: Send + 'static
{
    /// Creates a new, initially empty, thread coordinator.
    pub fn new() -> Self {
        let (tx, rx) = channel();

        Coordinator {
            receiver: rx,
            sender: tx,
            count: 0,
            count_active: 0,
        }
    }

    /// Adds a task to the thread coordinator which runs the provided closure,
    /// starting it immediately.
    pub fn spawn<F>(&mut self, f: F)
        where F: FnOnce() -> T,
              F: Send + 'static + UnwindSafe
    {

        let id = self.count;
        let sender = self.sender.clone();

        thread::spawn(move || {
            let res = catch_unwind(f);

            sender.send(Response {
                    id: id,
                    response: res,
                })
                .expect("`Coordinator` object hung up");
        });

        self.count += 1;
        self.count_active += 1;
    }

    /// If there are any running threads, waits until any of them either
    /// completes or panics. Returns either the result or panic value, then
    /// detaches all other running threads (but does _not_ stop them).
    ///
    /// If there are no running threads, returns `None` instantly.
    pub fn wait_next(&mut self) -> Option<thread::Result<T>> {
        if self.count_active == 0 {
            return None;
        }

        self.count_active -= 1;
        Some(self.receiver.recv().expect("`Coordinator` object hung up").response)
    }

    /// Waits until all running threads have either completed or panicked.
    /// Results (or panic values) are returned in the order the threads were
    /// spawned.
    pub fn wait_all(self) -> Vec<Option<thread::Result<T>>> {
        let mut responses: Vec<_> = (0..self.count).map(|_| None).collect();

        for resp in self.receiver.into_iter().take(self.count_active) {
            responses[resp.id] = Some(resp.response);
        }

        responses
    }
}

/// Iterate over thread responses in the order the threads finish. See the
/// documentation for `wait_next`.
impl<T> Iterator for Coordinator<T>
    where T: Send + 'static
{
    type Item = thread::Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.wait_next()
    }
}

struct Response<T> {
    id: usize,
    response: thread::Result<T>,
}
