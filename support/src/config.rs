use math::*;
use std::time::Duration;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Config {
    pub world_size: Vector2,
    pub max_bots: u32,
    pub fps: u32,
    pub ticks_per_step: u32,

    pub drive_friction: f64,
    pub thrust_limits: Clamped<f64>,
    pub turn_rate_limits: Clamped<f64>,
    pub gun_turn_rate_limits: Clamped<f64>,
    pub radar_turn_rate_limits: Clamped<f64>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            world_size: Vector2::new(100.0, 100.0),

            max_bots: 200,

            fps: 60,

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
    pub fn set_fps(&mut self, fps: u32) -> &mut Config {
        self.fps = fps;
        self
    }

    pub fn tick_duration(&self) -> Duration {
        let exact = 1.0 / self.fps as f64;
        let nanos = (exact * 1_000_000_000.0) as u32;
        Duration::new(0, nanos)
    }
}
