extern crate battlebots;
extern crate rand;
extern crate glium;

use glium::Surface;
use glium::glutin;

use battlebots::world::{World, TickLock};
use battlebots::ctl::user::*;
use battlebots::math::*;
use battlebots::config::*;
use battlebots::render::*;
use battlebots::threading::*;

use std::sync::Arc;
use std::process::{Command, Stdio};
use std::env;
use std::panic::AssertUnwindSafe;

fn main() {
    let config = Config::default();

    let ctls: Vec<Ctl> = (0..100).map(|id| {
        let child = Command::new(env::var("TESTPRG").unwrap())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let pos = Vector2::new(0.0, 0.0);

        Ctl::new(id, pos, config.clone(), child.stdin.unwrap(), child.stdout.unwrap())
    }).collect();

    let tick_lock = Arc::new(TickLock::new(ctls.len()));

    let world = World::new(config.clone(), ctls);

    // the coordinator for the individual robots
    let mut robo_coord = Coordinator::new();

    for robo in &world.all_robos {
        let robo = AssertUnwindSafe(robo.clone());
        let tick_lock = AssertUnwindSafe(tick_lock.clone());

        robo_coord.spawn(move|| {
            robo.run(&*tick_lock)
        });
    }

    let robos_data = world.robos_data.clone();

    // the coordinator for drawing, world running, and robots
    let mut main_coord = Coordinator::new();

    // Run the world in its own thread
    {
        let mut world = AssertUnwindSafe(world);
        let tick_lock = AssertUnwindSafe(tick_lock);

        main_coord.spawn(move|| {
            world.run(&*tick_lock).unwrap();
        });
    }

    // Do drawing in its own coordinated thread
    {
        main_coord.spawn(AssertUnwindSafe(move|| {
            use glium::DisplayBuild;

            let display = glutin::WindowBuilder::new()
                .build_glium()
                .unwrap();

            let mut draw_state = DrawState::new(&display, robos_data);

            loop {
                draw_state.update();

                let params = glium::DrawParameters {
                    multisampling: true,
                    smooth: Some(glium::Smooth::Nicest),
                    .. Default::default()
                };

                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 0.0);
                draw_state.draw(&mut target, &params).unwrap();
                target.finish().unwrap();

                for event in display.poll_events() {
                    match event {
                        glutin::Event::Closed => return,
                        _ => (),
                    }
                }
            }
        }));
    }

    {
        let robo_coord = AssertUnwindSafe(robo_coord);
        main_coord.spawn(move|| {
            // Keep going until any of the running robots stops with an error, and
            // report the error.
            robo_coord.0.wait_first().expect("Robot thread panicked").expect("Robo thread returned error");
        });
    }

    main_coord.wait_first().expect("Drawing, world or robots thread panicked");
}
