use config::*;
use math::*;
use rpc::*;

use serde_json;
use serde_json::error::Error as SerdeError;

use std::io;
use std::io::{BufRead, Write};
use std::ops::Deref;

/// A hook into the robot state
pub struct RoboHook {
    config: Config,
    state: BotState,
    resps: Vec<Response>,
}

#[allow(unused_variables)]
pub trait RoboController {
    /// Called just after the robot is created, before any simulation ticks.
    fn init(&mut self, hook: &mut RoboHook) {}

    /// Called periodically every few ticks (as determined by the
    /// configuration), allowing the robot to update itself over time.
    /// The time in seconds since the previous step is provided.
    fn step(&mut self, hook: &mut RoboHook, elapsed: f64) {}

    /// Called when an enemy robot is scanned.
    fn scan(&mut self, hook: &mut RoboHook, scan_pos: Vector2) {}

    /// Called when the robot is about to die, either because it was destroyed
    /// or because the simulation has ended. Note that an _immutable_ reference
    /// to the hook is provided, so the simulation can't be affected in any way
    /// by this method.
    fn kill(&mut self, hook: &RoboHook) {}
}

/// An error might can occur while controlling the robot.
#[derive(Debug)]
pub enum Error {
    Serialization(SerdeError),
    Deserialization(SerdeError),
    Write(io::Error),
    Read(io::Error),
}

impl RoboHook {
    /// View the configuration for this simulation.
    #[inline]
    pub fn config(&self) -> &Config { &self.config }

    /// Get the direction the gun is pointing, relative to the robot's body.
    #[inline]
    pub fn rel_gun_heading(&self) -> f64 {
        self.gun_heading - self.heading
    }

    /// Get the direction the radar is pointing, relative to the robot's body.
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

    /// Set the robot's turn rate. If outside the range specified by
    /// `turn_rate_limits` in the configuration, clamp to that range.
    #[inline]
    pub fn set_turn_rate(&mut self, turn_rate: f64) {
        let new_turn_rate = self.config().turn_rate_limits.clamp(turn_rate);

        self.state.turn_rate = new_turn_rate;
        self.resps.push(Response::SetTurnRate(new_turn_rate));
    }

    /// Set the robot's gun turn rate. If outside the range specified by
    /// `gun_turn_rate_limits` in the configuration, clamp to that range.
    #[inline]
    pub fn set_gun_turn_rate(&mut self, gun_turn_rate: f64) {
        let new_gun_turn_rate = self.config().gun_turn_rate_limits.clamp(gun_turn_rate);

        self.state.gun_turn_rate = new_gun_turn_rate;
        self.resps.push(Response::SetGunTurnRate(new_gun_turn_rate));
    }

    /// Set the robot's radar turn rate. If outside the range specified by
    /// `radar_turn_rate_limits` in the configuration, clamp to that range.
    #[inline]
    pub fn set_radar_turn_rate(&mut self, radar_turn_rate: f64) {
        let new_radar_turn_rate = self.config().radar_turn_rate_limits.clamp(radar_turn_rate);

        self.state.radar_turn_rate = new_radar_turn_rate;
        self.resps.push(Response::SetRadarTurnRate(new_radar_turn_rate));
    }

    /// Shoot a bullet with the given power, and consume that power. If outside
    /// the range specified by `bullet_power_limits` in the configuration, clamp
    /// to that range. Will cause an error if called more than once in a single
    /// step. Will do nothing if called when the robot's current shoot power is
    /// less than the provided power.
    #[inline]
    pub fn shoot(&mut self, power: f64) {
        let power = self.config().bullet_power_limits.clamp(power);
        self.resps.push(Response::Shoot(power));
    }

    /// Print the provided message to the simulation console.
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

    let mut alive = true;

    while alive {
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
            Step { elapsed } => ctl.step(&mut hook, elapsed),
            Scan { scan_pos } => ctl.scan(&mut hook, scan_pos),
            Kill => alive = false,
        }

        // Write responses
        let resp = try!(serde_json::to_string(&hook.resps).map_err(Error::Serialization));
        try!(writeln!(stdout, "{}", resp).map_err(Error::Write));
    }

    Ok(())
}
