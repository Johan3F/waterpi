#![allow(dead_code)]
use sysfs_gpio::Error;

use super::WaterPump;

pub struct WaterPumpMock {
    pub is_on: bool,
}

impl WaterPumpMock {
    pub fn new() -> WaterPumpMock {
        WaterPumpMock { is_on: false }
    }
}

impl WaterPump for WaterPumpMock {
    fn stop(&mut self) -> Result<(), Error> {
        self.is_on = false;
        Ok(())
    }

    fn on(&mut self) -> Result<(), Error> {
        self.is_on = true;
        Ok(())
    }

    fn off(&mut self) -> Result<(), Error> {
        self.is_on = false;
        Ok(())
    }
}
