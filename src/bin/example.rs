extern crate battlebots_support;
extern crate rand;

use battlebots_support::robo_controller::*;

struct Ctl {
    ticks: u32,
    reversing: bool,
}

impl RoboController for Ctl {
    fn init(&mut self, hook: &mut RoboHook) {
        hook.set_turn_rate(10.0);
        hook.set_gun_turn_rate(-10.0);
        hook.set_thrust(10.0);
    }

    fn step(&mut self, hook: &mut RoboHook, _elapsed: f64) {
        self.ticks -= 1;

        if self.ticks == 0 {
            self.ticks = mk_rand();
            self.reversing = !self.reversing;

            if self.reversing {
                hook.set_thrust(-10.0);
            } else {
                hook.set_thrust(10.0);
            }
        }
    }
}

fn main() {
    run(&mut Ctl { ticks: mk_rand(), reversing: false }).unwrap();
}

#[inline]
fn mk_rand() -> u32 {
    (rand::random::<u32>() % 20) + 16
}
