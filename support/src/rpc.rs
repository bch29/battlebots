use math::*;
use config::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BotState {
    /// Position, with the origin in the lower left.
    pub pos: Vector2,

    /// The direction that the robot is facing, in radians, going anticlockwise
    /// with 0 radians pointing right.
    pub heading: f64,

    /// The absolute direction that the gun is facing, in radians, going
    /// anticlockwise with 0 radians pointing right.
    pub gun_heading: f64,

    /// The absolute direction that the radar is facing, in radians, going
    /// anticlockwise with 0 radians pointing right.
    pub radar_heading: f64,

    /// The velocity, in units per second, in the direction the robot is facing.
    pub speed: f64,

    /// The acceleration, in units per second squared.
    pub thrust: f64,

    /// The rate of rotation of the robot.
    pub turn_rate: f64,

    /// The rate of rotation of the gun, relative to the robot.
    pub gun_turn_rate: f64,

    /// The rate of rotation of the radar, relative to the robot.
    pub radar_turn_rate: f64,

    /// The bot's hit points. When this reaches zero, the bot dies.
    pub hit_points: f64,

    /// Shooting requires shoot power. It regenerates over time.
    pub shoot_power: f64
}

/// A message which can be sent to the child process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// Sent on initialisation
    Init {
        /// The world's configuration.
        config: Config
    },

    /// Sent on each step.
    Step {
        /// The time since the last step (or since initialisation for the first
        /// step).
        elapsed: f64
    },

    /// Sent when an enemy robot is scanned.
    Scan {
        /// The location where the enemy was seen.
        scan_pos: Vector2
    },

    /// Sent when the robot dies (or the simulation ends).
    Kill,
}

/// A response which can be sent back to the robot to control it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    /// Set the thrust. Causes an error if the value is not within the range
    /// specified by `thrust_limits` in the configuration.
    SetThrust(f64),

    /// Set the turn rate. Causes an error if the value is not within the range
    /// specified by `turn_rate_limits` in the configuration.
    SetTurnRate(f64),

    /// Set the gun turn rate. Causes an error if the value is not within the
    /// range specified by `gun_turn_rate_limits` in the configuration.
    SetGunTurnRate(f64),

    /// Set the radar turn rate. Causes an error if the value is not within the
    /// range specified by `radar_turn_rate_limits` in the configuration.
    SetRadarTurnRate(f64),

    /// Fire a bullet in the direction the gun is currently heading, with the
    /// given power. Only one shoot command may be issued per frame.
    Shoot(f64),

    /// Print a message to the simulation console.
    DebugPrint(String),
}
