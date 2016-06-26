extern crate cgmath;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate glium;
extern crate crossbeam;

extern crate render_utils;
extern crate battlebots_support;

pub mod ctl;
pub mod robo;
pub mod world;
pub mod render;
pub mod threading;

pub use battlebots_support::math;
pub use battlebots_support::config;
pub use battlebots_support::rpc;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
