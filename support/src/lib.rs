#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate cgmath;
extern crate serde;
extern crate serde_json;

pub mod math;
pub mod config;
pub mod rpc;
pub mod robo_controller;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
