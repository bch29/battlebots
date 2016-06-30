use robo::Robo;
use ctl::RoboCtl;
use config::*;

use std::sync::{Arc, Mutex, RwLock, Barrier, RwLockReadGuard};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::Instant;
use std::fmt::Debug;

/// A world in which a robot battle takes place.
pub struct World<Ctl: RoboCtl> {
    all_robos: Vec<Arc<Robo<Ctl>>>,
    robos_data: Arc<Mutex<Vec<Ctl::PublicData>>>,
    tick_lock: Arc<TickLock>,
    config: Config,
    stop_receiver: Receiver<()>,
}

impl<Ctl: RoboCtl + Debug> World<Ctl> {
    /// Create a new world with the given configuration and robot controllers.
    /// The world will stop if it ever receives a message over the provided
    /// sender.
    pub fn new<Robos>(config: Config, robos: Robos) -> (Self, Arc<TickLock>, Sender<()>)
        where Robos: IntoIterator<Item = Ctl>
    {
        let all_robos: Vec<_> =
            robos.into_iter().map(|ctl| Arc::new(Robo::new(config.clone(), ctl))).collect();

        let (stop_sender, stop_receiver) = channel();

        let tick_lock = Arc::new(TickLock::new(all_robos.len()));

        (World {
            robos_data: Arc::new(Mutex::new(all_robos.iter()
                .map(|robo| robo.with_ctl(|ctl| ctl.public_data().clone()).unwrap())
                .collect())),

            all_robos: all_robos,
            config: config,
            stop_receiver: stop_receiver,
            tick_lock: tick_lock.clone(),
        },
         tick_lock,
         stop_sender)
    }

    /// Get the list of robots so they can be run.
    pub fn all_robos(&self) -> &[Arc<Robo<Ctl>>] {
        self.all_robos.as_slice()
    }

    /// Get the mutex-protected robot data so that it can be drawn.
    pub fn robos_data(&self) -> Arc<Mutex<Vec<Ctl::PublicData>>> {
        self.robos_data.clone()
    }

    /// Synchronously runs a world. Each of the contained robots must already be
    /// running independently and concurrently, or this will not make progress.
    ///
    /// Run until some error happens, or a stop message is received.
    pub fn run(&mut self) {
        // Initialise
        let tick_dur = self.config.tick_duration();

        let mut next_tick_time = Instant::now() + tick_dur;

        loop {
            // Do things safe in the knowledge that robots don't have locks on
            // their own state
            {
                let mut robos_data = self.robos_data.lock().unwrap();

                *robos_data = self.all_robos
                    .iter()
                    .map(|robo| robo.with_ctl(|ctl| ctl.public_data().clone()).unwrap())
                    .collect();
            }

            // Allow robots to make progress
            if let Some(mut tick_guard) = self.tick_lock.take() {
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
                return;
            }
        }
    }
}

// =============================================================================
//  Tick locks. Very hairy synchronisation, probably don't touch this.
// =============================================================================

/// A lock object used to coordinate the world's and the robots' ticks, so that
/// none gets ahead of any others.
pub struct TickLock {
    barrier: Barrier,
    is_running: RwLock<bool>,
}

/// An RAII guard indicating that we are in the process of ticking. The tick
/// ends when the `TickGuard` is dropped.
pub struct TickGuard<'a> {
    barrier: &'a Barrier,

    // A reference to the `is_running` lock is kept so that a thread can stop
    // the world via the guard.
    is_running: &'a RwLock<bool>,

    // A guard on the `is_running` lock is kept so that the world can't
    // be stopped while some threads are still ticking.
    running_guard: Option<RwLockReadGuard<'a, bool>>,
}

impl TickLock {
    fn new(size: usize) -> Self {
        TickLock {
            barrier: Barrier::new(size + 1),
            is_running: RwLock::new(true),
        }
    }

    /// Try to take the lock. If the world is running, wait until we are allowed
    /// to tick and return the RAII guard. Otherwise, return `None`.
    pub fn take(&self) -> Option<TickGuard> {

        // First check that the world is running and return straight away if
        // not. Importantly, keep hold of the lock to make sure the world can't
        // be stopped before this tick is complete.
        let running = self.is_running.read().unwrap();
        if !*running {
            return None;
        }

        // Wait until all threads are at this point before allowing any to have
        // the guard.
        self.barrier.wait();

        Some(TickGuard {
            barrier: &self.barrier,
            is_running: &self.is_running,
            running_guard: Some(running),
        })
    }
}

impl<'a> TickGuard<'a> {
    /// Stop the world. This will wait until all other threads are done with
    /// their current `TickGuard`s, then make sure no `TickGuard`s can be taken
    /// in the future. This is private because only the world can stop itself.
    fn stop(&mut self) {
        // Drop the running read guard so that the write lock can be taken.
        self.running_guard = None;

        // Take the write lock on `is_running`. Due to the presence of
        // `running_guard` in each `TickGuard`, this also waits until all
        // `TickGuard`s but this one have been dropped, and hence all other
        // threads have finished ticking.
        let mut running = self.is_running.write().unwrap();
        *running = false;
    }
}

impl<'a> Drop for TickGuard<'a> {
    fn drop(&mut self) {
        // Make sure the running guard is dropped first so we don't block a
        // thread that wants to stop the world.
        self.running_guard = None;
        self.barrier.wait();
    }
}
