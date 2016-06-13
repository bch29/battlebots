#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]
#![feature(mpsc_select)]

extern crate cgmath;
extern crate serde;
extern crate serde_json;

pub mod robo;
// pub mod std_robo;
// pub mod external_robo;
// pub mod robo_controller;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
