extern crate robo2;
extern crate cgmath;
extern crate schedule_recv;

use robo2::std_robo::*;
use robo2::external_robo::*;
use cgmath::{Zero, Vector2};

use std::env;
use std::process::Command;

use schedule_recv::periodic_ms;

fn send_event(robo: &ExternalRobo, ev: Event) {
    robo.send_cmd(Cmd::SysCmd(SysCmd::RaiseEvent(ev))).unwrap();
}

fn main() {
    let st = StdRobo {
        health: 100.0,
        pos: SerVector2(Vector2::new(0.0, 0.0)),
        vel: SerVector2(Vector2::zero()),
    };

    let testprg = env::var_os("TESTPRG").expect("TESTPRG not defined");
    let robo = ExternalRobo::new(st, Command::new(testprg)).unwrap();

    send_event(&robo, Event::Init);

    let tick = periodic_ms(1);

    loop {
        tick.recv().unwrap();
        send_event(&robo, Event::Tick);
    }

    // robo.wait();
}
