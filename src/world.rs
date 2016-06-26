use robo::Robo;
use ctl::RoboCtl;
use config::*;

use std::sync::{Arc, Mutex, RwLock, Barrier, RwLockReadGuard};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::Instant;

#[derive(Debug)]
pub enum Error {
    Poisoned
}

pub struct World<Ctl: RoboCtl> {
    pub all_robos: Vec<Arc<Robo<Ctl>>>,
    pub robos_data: Arc<Mutex<Vec<Ctl::PublicData>>>,
    config: Config,
    ticks_done: u64,
    stop_receiver: Receiver<()>,
}

impl<Ctl: RoboCtl> World<Ctl> {
    /// Create a new world with the given configuration and robot controllers.
    /// The world will stop if it ever receives a message over the provided
    /// sender.
    pub fn new<Robos>(config: Config, robos: Robos) -> (Self, Sender<()>)
        where Robos: IntoIterator<Item = Ctl>
    {
        let all_robos: Vec<_> =
            robos.into_iter().map(|ctl| Arc::new(Robo::new(config.clone(), ctl))).collect();

        let (stop_sender, stop_receiver) = channel();

        (World {
            robos_data: Arc::new(Mutex::new(all_robos.iter().map(|robo| {
                robo.with_ctl(|ctl| ctl.public_data()).unwrap()
            }).collect())),

            all_robos: all_robos,

            config: config.clone(),

            ticks_done: 0,

            stop_receiver: stop_receiver,
        }, stop_sender)
    }

    /// Synchronously runs a world. Each of the contained robots must already be
    /// running independently and concurrently, and the provided `TickLock`'s
    /// size must equal the number of robots, or this will not make progress.
    /// Run until some error happens, or a stop message is received.
    pub fn run(&mut self, tick_lock: &TickLock) -> Result<(), Error> {
        // Initialise
        let tick_dur = self.config.tick_duration();

        let mut next_tick_time = Instant::now() + tick_dur;

        loop {
            // Do things safe in the knowledge that robots don't have locks on
            // their own state
            {
                let mut robos_data = self.robos_data.lock().unwrap();

                *robos_data = self.all_robos.iter().map(|robo| {
                    robo.with_ctl(|ctl| ctl.public_data()).unwrap()
                }).collect();
            }

            // Allow robots to make progress
            if let Some(mut tick_guard) = tick_lock.take() {
                self.ticks_done += 1;

                if let Ok(_) = self.stop_receiver.try_recv() {
                    tick_guard.stop();
                }

                // Handle timing. Current implementation will never catch back up after
                // losing frames. Consider implementing that.
                let now_time = Instant::now();
                if now_time < next_tick_time {
                    thread::sleep(next_tick_time - now_time);
                }
                next_tick_time += tick_dur;
            } else {
                println!("World exiting");
                return Ok(())
            }
        }
    }
}

pub struct TickLock {
    start_bar: Barrier,
    end_bar: Barrier,
    is_running: RwLock<bool>,
}

pub struct TickGuard<'a> {
    end_bar: &'a Barrier,
    is_running: &'a RwLock<bool>,
    running_guard: Option<RwLockReadGuard<'a, bool>>,
}

impl TickLock {
    pub fn new(size: usize) -> Self {
        TickLock {
            start_bar: Barrier::new(size + 1),
            end_bar: Barrier::new(size + 1),
            is_running: RwLock::new(true),
        }
    }

    /// Try to take the lock. If the world is running, wait until we are allowed
    /// to tick and return the guard. Otherwise, return `None`.
    pub fn take(&self) -> Option<TickGuard> {

        let running = self.is_running.read().unwrap();
        if !*running { return None }

        self.start_bar.wait();

        Some(TickGuard {
            end_bar: &self.end_bar,
            is_running: &self.is_running,
            running_guard: Some(running),
        })
    }

    /// Stop the world. This will always block until no thread has a guard on
    /// this lock (and hence will cause a deadlock if called by a thread that
    /// already has a guard on this lock).
    pub fn stop_world(&self) {
        println!("Waiting for stop lock");
        let mut running = self.is_running.write().unwrap();
        println!("Acquired stop lock");
        *running = false;
    }
}

impl<'a> TickGuard<'a> {
    fn stop(&mut self) {
        self.running_guard = None;

        let mut running = self.is_running.write().unwrap();
        *running = false;
    }
}

impl<'a> Drop for TickGuard<'a> {
    fn drop(&mut self) {
        // Make sure the running guard is dropped first so we don't block a
        // thread that wants to stop the world.
        self.running_guard = None;
        self.end_bar.wait();
    }
}
