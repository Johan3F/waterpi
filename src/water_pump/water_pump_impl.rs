use sysfs_gpio::{Direction, Error, Pin};

use crate::metrics::*;

use super::WaterPump;

pub struct WaterPumpImpl {
    pump: Pin,
}

impl WaterPumpImpl {
    pub fn new(pin: u64) -> Result<WaterPumpImpl, Error> {
        let water_pump = Pin::new(pin);
        water_pump.export()?;
        water_pump.set_direction(Direction::Out)?;
        water_pump.set_value(0)?;
        PUMP_ON.set(0.0);

        Ok(WaterPumpImpl { pump: water_pump })
    }
}

impl WaterPump for WaterPumpImpl {
    fn stop(&mut self) -> Result<(), Error> {
        self.off()
    }

    fn on(&mut self) -> Result<(), Error> {
        PUMP_ON.set(1.0);
        self.pump.set_value(1)
    }

    fn off(&mut self) -> Result<(), Error> {
        PUMP_ON.set(0.0);
        self.pump.set_value(0)
    }
}
