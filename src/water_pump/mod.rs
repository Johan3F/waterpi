mod water_pump_impl;
mod water_pump_mock;

pub use water_pump_impl::WaterPumpImpl;
pub use water_pump_mock::WaterPumpMock;

use sysfs_gpio::Error;

pub trait WaterPump: Send {
    fn stop(&mut self) -> Result<(), Error>;
    fn on(&mut self) -> Result<(), Error>;
    fn off(&mut self) -> Result<(), Error>;
}
