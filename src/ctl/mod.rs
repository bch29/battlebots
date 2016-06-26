use std::time::Duration;

pub mod user;

pub trait RoboCtl {
    type PublicData: Send + Clone + 'static;
    type Error: Send + 'static;

    fn init(&mut self) -> Result<(), Self::Error>;

    fn tick(&mut self, elapsed: Duration) -> Result<(), Self::Error>;

    fn public_data(&self) -> Self::PublicData;
}
