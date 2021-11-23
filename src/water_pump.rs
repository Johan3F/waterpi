use sysfs_gpio::{Direction, Error, Pin};

pub struct WaterPump {
    pump: Pin,
}

impl WaterPump {
    pub fn new(pin: u64) -> Result<WaterPump, Error> {
        let water_pump = Pin::new(pin);
        water_pump.export()?;
        water_pump.set_direction(Direction::Out)?;
        water_pump.set_value(0)?;

        Ok(WaterPump { pump: water_pump })
    }
    pub fn stop(&self) -> Result<(), Error> {
        self.off()
    }

    pub fn on(&self) -> Result<(), Error> {
        self.pump.set_value(1)
    }

    pub fn off(&self) -> Result<(), Error> {
        self.pump.set_value(0)
    }
}
