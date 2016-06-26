use math::*;
use config::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BotState {
    pub pos: Vector2,
    pub heading: f64,
    pub gun_heading: f64,
    pub radar_heading: f64,
    pub speed: f64,
    pub thrust: f64,
    pub turn_rate: f64,
    pub gun_turn_rate: f64,
    pub radar_turn_rate: f64,
}

/// A message which can be sent to the child process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Init { config: Config },
    Step { elapsed: f64 },
    Scan { scan_pos: Vector2 },
    Kill,
}

/// A response which can be received from the child process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    SetThrust(f64),
    SetTurnRate(f64),
    SetGunTurnRate(f64),
    SetRadarTurnRate(f64),

    DebugPrint(String),
}
