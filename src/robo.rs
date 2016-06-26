use config::Config;
use world::TickLock;
use ctl::RoboCtl;

use std::sync::{Arc, Mutex, PoisonError, MutexGuard};
use std::thread;
use std::time::Instant;

/// An asynchronous robot, whose behaviour is determined by the `Ctl` type.
pub struct Robo<Ctl: RoboCtl> {
    #[allow(dead_code)]
    config: Config,
    state: Mutex<Ctl>,
}

#[derive(Debug)]
pub enum Error<Ctl>
    where Ctl: RoboCtl
{
    StatePoisoned,
    Ctl(Ctl::Error),
}

impl<Ctl: RoboCtl> Robo<Ctl> {
    /// Creates a new robot in the given world. Requires taking a write lock on
    /// the world and thus may fail if the world's lock in poisoned.
    pub fn new(config: Config, ctl: Ctl) -> Self {
        Robo {
            config: config,
            state: Mutex::new(ctl),
        }
    }

    /// Synchronously runs the robot. Should only be called once (from any
    /// thread) and should usually be run in a new thread. See `rob::run_async`.
    pub fn run(&self, tick_lock: &TickLock) -> Result<(), Error<Ctl>> {
        // Initialise in a block to make sure to drop the lock on the state when
        // done.
        {
            // Get a lock on the state
            let mut state = try!(self.state.lock());

            try!(state.init().map_err(Error::Ctl));
        }

        let mut prev_time = Instant::now();

        loop {
            // Wait until we are allowed to tick
            if let Some(_tick_guard) = tick_lock.take() {
                // Get a lock on the state
                let mut state = try!(self.state.lock());

                // Tell the `Ctl` to tick
                let now = Instant::now();
                try!(state.tick(now.duration_since(prev_time)).map_err(Error::Ctl));
                prev_time = now;
            } else {
                // Get a lock on the state
                let mut state = try!(self.state.lock());

                try!(state.kill().map_err(Error::Ctl));
                return Ok(());
            }
        }
    }

    /// Do something with the underlying `Ctl` object.
    pub fn with_ctl<F, R>(&self, f: F) -> Result<R, Error<Ctl>>
        where F: FnOnce(&Ctl) -> R
    {

        self.state.lock().map(|ctl| f(&*ctl)).map_err(Into::into)
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
    pub fn run_async(robo: Arc<Robo<Ctl>>,
                     tick_lock: Arc<TickLock>)
                     -> thread::JoinHandle<Result<(), Error<Ctl>>>
        where Ctl: Send + 'static
    {

        thread::spawn(move || robo.run(&*tick_lock))
    }
}

// Can't implement From<Ctl::Error> without specialisation.
impl<'a, Ctl> From<PoisonError<MutexGuard<'a, Ctl>>> for Error<Ctl>
    where Ctl: RoboCtl
{
    fn from(_: PoisonError<MutexGuard<Ctl>>) -> Self {
        Error::StatePoisoned
    }
}
