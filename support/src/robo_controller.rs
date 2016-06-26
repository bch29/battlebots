use config::*;
use math::*;
use rpc::*;

use serde_json;
use serde_json::error::Error as SerdeError;

use std::io;
use std::io::{BufRead, Write};
use std::ops::Deref;

pub struct RoboHook {
    config: Config,
    state: BotState,
    resps: Vec<Response>,
}

#[allow(unused_variables)]
pub trait RoboController {
    fn init(&mut self, hook: &mut RoboHook) {}
    fn tick(&mut self, hook: &mut RoboHook, elapsed: f64) {}
    fn scan(&mut self, hook: &mut RoboHook, scan_pos: Vector2) {}
}

#[derive(Debug)]
pub enum Error {
    Serialization(SerdeError),
    Deserialization(SerdeError),
    Write(io::Error),
    Read(io::Error),
}

impl RoboHook {
    pub fn config(&self) -> &Config { &self.config }

    #[inline]
    pub fn rel_gun_heading(&self) -> f64 {
        self.gun_heading - self.heading
    }

    #[inline]
    pub fn rel_radar_heading(&self) -> f64 {
        self.radar_heading - self.heading
    }

    /// Set the robot's thrust (acceleration). If outside the range specified by
    /// `thrust_limits` in the configuration, clamp to that range.
    #[inline]
    pub fn set_thrust(&mut self, thrust: f64) {
        let new_thrust = self.config().thrust_limits.clamp(thrust);

        self.state.thrust = new_thrust;
        self.resps.push(Response::SetThrust(new_thrust));
    }

    #[inline]
    pub fn set_turn_rate(&mut self, turn_rate: f64) {
        let new_turn_rate = self.config().turn_rate_limits.clamp(turn_rate);

        self.state.turn_rate = new_turn_rate;
        self.resps.push(Response::SetTurnRate(new_turn_rate));
    }

    #[inline]
    pub fn set_gun_turn_rate(&mut self, gun_turn_rate: f64) {
        let new_gun_turn_rate = self.config().gun_turn_rate_limits.clamp(gun_turn_rate);

        self.state.gun_turn_rate = new_gun_turn_rate;
        self.resps.push(Response::SetGunTurnRate(new_gun_turn_rate));
    }

    #[inline]
    pub fn set_radar_turn_rate(&mut self, radar_turn_rate: f64) {
        let new_radar_turn_rate = self.config().radar_turn_rate_limits.clamp(radar_turn_rate);

        self.state.radar_turn_rate = new_radar_turn_rate;
        self.resps.push(Response::SetRadarTurnRate(new_radar_turn_rate));
    }

    #[inline]
    pub fn debug_print(&mut self, msg: &str) {
        self.resps.push(Response::DebugPrint(msg.to_owned()));
    }
}

impl Deref for RoboHook {
    type Target = BotState;

    fn deref(&self) -> &BotState {
        &self.state
    }
}

pub fn run<Ctl: RoboController>(ctl: &mut Ctl) -> Result<(), Error> {
    let stdin = io::stdin();
    let stdout = io::stdout();

    let mut stdin = stdin.lock();
    let mut stdout = stdout.lock();

    let mut config = Config::default();

    loop {
        // Read a message and act on it
        let mut buf = String::new();
        try!(stdin.read_line(&mut buf).map_err(Error::Read));
        let (state, msg): (BotState, Message) = try!(serde_json::from_str(buf.as_str()).map_err(Error::Deserialization));

        let mut hook = RoboHook {
            config: config.clone(),
            state: state,
            resps: Vec::new(),
        };

        use rpc::Message::*;

        match msg {
            Init { config: new_config } => {
                config = new_config;
                hook.config = config.clone();
                ctl.init(&mut hook)
            },
            Tick { elapsed } => ctl.tick(&mut hook, elapsed),
            Scan { scan_pos } => ctl.scan(&mut hook, scan_pos),
        }

        // Write responses
        let resp = try!(serde_json::to_string(&hook.resps).map_err(Error::Serialization));
        try!(writeln!(stdout, "{}", resp).map_err(Error::Write));
    }
}
