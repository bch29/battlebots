use std::time::Duration;
use std::fmt::Debug;

pub mod user;

/// A controller for a robot.
pub trait RoboCtl {
    /// Type for internal robot data that is available to the world and renderer.
    type PublicData: Send + Clone + 'static;

    /// Type for errors that might be thrown by callbacks.
    type Error: Debug + Send + 'static;

    /// Called when the robot is first initialised, before any ticks.
    fn init(&mut self) -> Result<(), Self::Error>;

    /// Called on each simulation tick. The elapsed time since the last tick (or
    /// initialisation) is provided.
    fn tick(&mut self, elapsed: Duration) -> Result<(), Self::Error>;

    /// Called when the robot needs to die, either because it has been destroyed
    /// or the simulation is over.
    fn kill(&mut self) -> Result<(), Self::Error>;

    /// Fetch the public data.
    fn public_data(&self) -> &Self::PublicData;
}
