use math::*;
use config::*;
use ctl::*;
use rpc::*;

use std::time::Duration;
use std::sync::Arc;
use std::io::{BufReader, Read, Write};
use std::thread;
use std::fmt;

mod process;

use self::process::*;

/// Controller for a user's robot, based on an external process.
pub struct Ctl {
    id: u64,
    ticks_until_step: u32,
    elapsed_since_step: f64,

    next_shot_power: Option<f64>,

    state: BotState,
    config: Config,

    relay: Arc<Relay>,
}

impl fmt::Debug for Ctl {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f,
               "Ctl {{ id: {}, ticks_until_steps: {}, state: {:?}, config: {:?}, relay: ?, \
                join_handle: ? }}",
               self.id,
               self.ticks_until_step,
               self.state,
               self.config)
    }
}

#[derive(Debug)]
pub enum Error {
    Process(ProcessError),
    SlowResponse,

    BadThrust(f64),
    BadTurnRate(f64),
    BadGunTurnRate(f64),
    BadRadarTurnRate(f64),
    TooManyBulletsPerFrame,
}

impl Ctl {
    pub fn new<R, W>(id: u64,
                     initial_pos: Vector2,
                     config: Config,
                     output_writer: W,
                     input_reader: R)
                     -> Self
        where R: Read + Send + 'static,
              W: Write + Send + 'static {

        let (process, relay) = Process::new(output_writer, BufReader::new(input_reader));

        thread::spawn(move || process.run());

        Ctl {
            id: id,
            ticks_until_step: config.ticks_per_step,
            elapsed_since_step: 0.0,
            next_shot_power: None,

            state: BotState { pos: initial_pos, ..BotState::default() },
            config: config,

            relay: relay,
        }
    }

    /// Perform the effects of a single response.
    fn apply_resp(&mut self, resp: Response) -> Result<(), Error> {
        use rpc::Response::*;
        use self::Error::*;

        match resp {
            SetThrust(x) => {
                self.state.thrust = try!(self.config.thrust_limits.check(x).map_err(BadThrust))
            }

            SetTurnRate(x) => {
                self.state.turn_rate =
                    try!(self.config.turn_rate_limits.check(x).map_err(BadTurnRate))
            }

            SetGunTurnRate(x) => {
                self.state.gun_turn_rate =
                    try!(self.config.gun_turn_rate_limits.check(x).map_err(BadGunTurnRate))
            }

            SetRadarTurnRate(x) => {
                self.state.radar_turn_rate =
                    try!(self.config.radar_turn_rate_limits.check(x).map_err(BadRadarTurnRate))
            }

            Shoot(power) => {
                if self.next_shot_power.is_none() {
                    self.next_shot_power = Some(power);
                } else {
                    return Err(TooManyBulletsPerFrame)
                }
            }

            DebugPrint(msg) => println!("Bot {}: {}", self.id, msg),
        }

        Ok(())
    }
}

impl RoboCtl for Ctl {
    type PublicData = BotState;
    type Error = Error;

    fn init(&mut self) -> Result<(), Error> {
        self.relay.send_msg((self.state.clone(), Message::Init { config: self.config.clone() }));

        Ok(())
    }

    fn tick(&mut self, elapsed: Duration) -> Result<(), Error> {
        let elapsed = duration_float(elapsed);

        // Deal with external stepping. We use an asynchronous `Process` for
        // this so that we don't block the simulation while the robot is doing
        // its calculations for this step.
        self.ticks_until_step -= 1;
        self.elapsed_since_step += elapsed;

        if self.ticks_until_step == 0 {
            self.ticks_until_step = self.config.ticks_per_step;

            if let Some(resps) = self.relay.try_recv_resps() {
                for resp in resps {
                    try!(self.apply_resp(resp))
                }

                self.relay.send_msg((self.state.clone(),
                                     Message::Step { elapsed: self.elapsed_since_step }))
            } else {
                // TODO: Consider making this optionally not throw an error, or
                // maybe only throw after blocking for longer.
                return Err(Error::SlowResponse);
            }
        }

        // Do the simulation
        self.state.heading += self.state.turn_rate * elapsed;
        self.state.gun_heading += (self.state.gun_turn_rate + self.state.turn_rate) * elapsed;
        self.state.radar_heading += (self.state.radar_turn_rate + self.state.turn_rate) * elapsed;
        self.state.speed += self.state.thrust * elapsed;
        self.state.speed *= self.config.drive_friction;

        let dir = Vector2::new(self.state.heading.cos(), self.state.heading.sin());
        self.state.pos += dir * self.state.speed * elapsed;

        Ok(())
    }

    fn kill(&mut self) -> Result<(), Error> {
        self.relay.send_msg((self.state.clone(), Message::Kill));

        Ok(())
    }

    fn public_data(&self) -> &BotState {
        &self.state
    }
}

fn duration_float(d: Duration) -> f64 {
    d.as_secs() as f64 + (d.subsec_nanos() / 1000u32) as f64 / 1000000.0
}
