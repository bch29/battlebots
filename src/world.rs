use math::*;
use robo::Robo;
use ctl::RoboCtl;
use config::*;

use std::sync::{Arc, Mutex, Barrier};
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
    size: Vector2,
    ticks_done: u64,
}

impl<Ctl: RoboCtl> World<Ctl> {
    pub fn new<Robos>(config: Config, robos: Robos) -> Self
        where Robos: IntoIterator<Item = Ctl>
    {
        let all_robos: Vec<_> =
            robos.into_iter().map(|ctl| Arc::new(Robo::new(config.clone(), ctl))).collect();

        World {
            robos_data: Arc::new(Mutex::new(all_robos.iter().map(|robo| {
                robo.with_ctl(|ctl| ctl.public_data()).unwrap()
            }).collect())),

            all_robos: all_robos,

            config: config.clone(),

            size: config.world_size,
            ticks_done: 0,
        }
    }

    /// Synchronously runs a world which is protected by an `Mutex`. Each of the
    /// contained robots must be running independently and concurrently, and the
    /// provided `TickLock`'s size must equal the number of robots, or this will not
    /// make progress.
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

            {
                // Allow robots to make progress
                let _tick_guard = tick_lock.take();

                self.ticks_done += 1;

                // Handle timing. Current implementation will never catch back up after
                // losing frames. Consider implementing that.
                let now_time = Instant::now();
                if now_time < next_tick_time {
                    thread::sleep(next_tick_time - now_time);
                }
                next_tick_time += tick_dur;
            }
        }
    }
}

pub struct TickLock {
    start_bar: Barrier,
    end_bar: Barrier,
}

pub struct TickGuard<'a> {
    end_bar: &'a Barrier,
}

impl TickLock {
    pub fn new(size: usize) -> Self {
        TickLock {
            start_bar: Barrier::new(size + 1),
            end_bar: Barrier::new(size + 1),
        }
    }

    pub fn take(&self) -> TickGuard {
        self.start_bar.wait();

        TickGuard {
            end_bar: &self.end_bar
        }
    }
}

impl<'a> Drop for TickGuard<'a> {
    fn drop(&mut self) {
        self.end_bar.wait();
    }
}
