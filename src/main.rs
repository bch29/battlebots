extern crate battlebots;
extern crate rand;
extern crate glium;

use glium::Surface;
use glium::glutin;

use battlebots::world::World;
use battlebots::ctl::user::Ctl;
use battlebots::math::Vector2;
use battlebots::config::Config;
use battlebots::render::DrawState;
use battlebots::threading::Coordinator;

use std::env;
use std::process::{Command, Stdio};
use std::panic::AssertUnwindSafe;

fn main() {
    // TODO: load from disk or environment variables?
    let config = Config::default();

    let num_bots = 100;

    println!("Starting robot processes...");

    // Create some robots from external processes.
    let ctls: Vec<Ctl> = (0..num_bots)
        .map(|id| {
            let child = Command::new(env::var("TESTPRG").unwrap())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();

            let pos = Vector2::new(0.0, 0.0);

            Ctl::new(id,
                     pos,
                     config.clone(),
                     child.stdin.unwrap(),
                     child.stdout.unwrap())
        })
        .collect();

    println!("Starting the simulation...");

    let (world, tick_lock, stop_world) = World::new(config.clone(), ctls);

    // the coordinator for the individual robots
    let mut robo_coord = Coordinator::new();

    // Spawn each robot in its own coordinated thread
    for robo in world.all_robos() {
        let robo = AssertUnwindSafe(robo.clone());
        let tick_lock = AssertUnwindSafe(tick_lock.clone());

        robo_coord.spawn(move || robo.run(&*tick_lock));
    }

    // This is a synchronised view into the current state of the robots, to give
    // to the drawing thread.
    let robos_data = world.robos_data();

    // the coordinator for drawing, world running, and robots
    let mut main_coord = Coordinator::new();

    // Run the world in its own coordinated thread
    {
        let mut world = AssertUnwindSafe(world);

        main_coord.spawn(move || world.run());
    }

    // Do drawing in its own coordinated thread
    {
        main_coord.spawn(AssertUnwindSafe(move || {
            use glium::DisplayBuild;

            let display = glutin::WindowBuilder::new()
                .with_vsync()
                .build_glium()
                .unwrap();

            let mut draw_state = DrawState::new(&display, robos_data);

            loop {
                draw_state.update();

                let params = glium::DrawParameters {
                    multisampling: true,
                    smooth: Some(glium::Smooth::Nicest),
                    ..Default::default()
                };

                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 0.0);
                draw_state.draw(&mut target, &params).unwrap();
                target.finish().unwrap();

                for event in display.poll_events() {
                    if let glutin::Event::Closed = event {
                        return;
                    }
                }
            }
        }));
    }

    // Link the coordinated robots to the main thread coordinator
    {
        let robo_coord = AssertUnwindSafe(robo_coord);

        main_coord.spawn(move || {
            // Keep going until the running robots stop. If they stop with an
            // error, panic and report it.
            for res in robo_coord.0 {
                res.expect("Robot thread panicked").expect("Robo thread returned error");
            }
        });
    }

    // Wait for the first of the drawing, world or robots thread to end. If
    // there are no errors, this will always be the drawing thread (when the
    // user closes the window).
    main_coord.next().unwrap().expect("Drawing, world or robots thread panicked");

    // If the world hasn't already stopped (which it shouldn't have unless there
    // was a panic), tell it to stop. It will in turn tell each of the robots to
    // stop.
    let _ = stop_world.send(());

    // Wait for other running threads to finish up.
    for res in main_coord {
        res.expect("Panic at shutdown.");
    }

    println!("Goodbye!");
}
