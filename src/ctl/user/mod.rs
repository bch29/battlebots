use math::*;
use config::*;
use ctl::*;
use rpc::*;

use std::time::Duration;
use std::sync::Arc;
use std::io::{BufReader, Read, Write};
use std::thread;

pub mod process;

use self::process::*;

pub struct Ctl {
    id: u64,
    ticks_until_step: u32,
    state: BotState,
    config: Config,

    relay: Arc<Relay>,
}

use std::fmt;

impl fmt::Debug for Ctl {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Ctl {{ id: {}, ticks_until_steps: {}, state: {:?}, config: {:?}, relay: ?, join_handle: ? }}",
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
}

impl Ctl {
    pub fn new<R, W>(id: u64, initial_pos: Vector2, config: Config, output_writer: W, input_reader: R) -> Self
        where R: Read + Send + 'static, W: Write + Send + 'static {

        let process = Process::new(output_writer, BufReader::new(input_reader));
        let relay = process.relay();

        thread::spawn(move|| {
            process.run()
        });

        Ctl {
            id: id,
            ticks_until_step: config.ticks_per_step,

            state: BotState {
                pos: initial_pos,
                .. BotState::default()
            },

            config: config,

            relay: relay,
        }
    }

    fn apply_resp(&mut self, resp: Response) -> Result<(), Error> {
        use rpc::Response::*;
        use self::Error::*;

        match resp {
            SetThrust(x) => self.state.thrust =
                try!(self.config.thrust_limits.check(x).map_err(BadThrust)),
            SetTurnRate(x) => self.state.turn_rate =
                try!(self.config.turn_rate_limits.check(x).map_err(BadTurnRate)),
            SetGunTurnRate(x) => self.state.gun_turn_rate =
                try!(self.config.gun_turn_rate_limits.check(x).map_err(BadGunTurnRate)),
            SetRadarTurnRate(x) => self.state.radar_turn_rate =
                try!(self.config.radar_turn_rate_limits.check(x).map_err(BadRadarTurnRate)),

            DebugPrint(msg) => println!("Bot {}: {}", self.id, msg),
        }

        Ok(())
    }
}

impl RoboCtl for Ctl {
    type PublicData = BotState;
    type Error = Error;

    fn init(&mut self) -> Result<(), Error> {
        self.relay.send_msg((
            self.state.clone(),
            Message::Init {
                config: self.config.clone(),
            }));

        Ok(())
    }

    fn tick(&mut self, elapsed: Duration) -> Result<(), Error> {
        let elapsed = duration_float(elapsed);

        // Deal with external stepping
        self.ticks_until_step -= 1;
        if self.ticks_until_step == 0 {
            self.ticks_until_step = self.config.ticks_per_step;

            if let Some(resps) = self.relay.try_recv_resps() {
                for resp in resps {
                    try!(self.apply_resp(resp))
                }

                self.relay.send_msg((
                    self.state.clone(),
                    Message::Tick {
                        elapsed: elapsed,
                    }))
            } else {
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

    fn public_data(&self) -> BotState {
        self.state.clone()
    }
}

fn duration_float(d: Duration) -> f64 {
    d.as_secs() as f64 + (d.subsec_nanos() / 1000u32) as f64 / 1000000.0
}
