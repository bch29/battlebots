use math::*;
use std::time::Duration;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Config {

    /// The size of the world.
    pub world_size: Vector2,

    /// The frame rate of the simulation (but not necessarily the rendering).
    pub ticks_per_second: u32,

    /// The number of ticks between each external step.
    pub ticks_per_step: u32,

    /// The multiplicative friction per tick for robots.
    pub drive_friction: f64,

    /// The range of allowed thrust values.
    pub thrust_limits: Clamped<f64>,

    /// The range of allowed turn rate values.
    pub turn_rate_limits: Clamped<f64>,

    /// The range of allowed gun turn rate values.
    pub gun_turn_rate_limits: Clamped<f64>,

    /// The range of allowed radar turn rate values.
    pub radar_turn_rate_limits: Clamped<f64>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            world_size: Vector2::new(100.0, 100.0),

            ticks_per_second: 60,

            ticks_per_step: 5,

            drive_friction: 0.95,
            thrust_limits: Clamped::new(-10.0, 10.0),
            turn_rate_limits: Clamped::new(-2.0, 2.0),
            gun_turn_rate_limits: Clamped::new(-2.0, 2.0),
            radar_turn_rate_limits: Clamped::new(-2.0, 2.0),
        }
    }
}

impl Config {
    /// Calculate the length of a tick as a `Duration` based on the `ticks_per_second`.
    pub fn tick_duration(&self) -> Duration {
        let exact = 1.0 / self.ticks_per_second as f64;
        let nanos = (exact * 1_000_000_000.0) as u32;
        Duration::new(0, nanos)
    }
}
