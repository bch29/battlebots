extern crate robo2;
extern crate cgmath;
extern crate serde_json;

use robo2::std_robo::*;
use robo2::robo_controller::*;

use cgmath::vec2;

struct Ctl;

impl RoboController for Ctl {
    fn handle_event(&mut self, hook: &RoboHook, event: Event) {
        match event {
            Event::Init => {
                hook.set_vel(vec2(1.0, 1.0));
                hook.log("Initialising!");
            }
            Event::Tick => {
                let st = hook.get_state();
                let msg = format!("Got tick, state is: {:?}", st);
                hook.log(msg);
            }
        }
    }
}

fn main() {
    let robo = RemoteRobo::new();
    let mut ctl = Ctl;
    robo.run_controller(&mut ctl);
}
