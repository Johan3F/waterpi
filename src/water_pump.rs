use sysfs_gpio::{Direction, Error, Pin};

pub trait WaterPump: Send {
    fn stop(&mut self) -> Result<(), Error>;
    fn on(&mut self) -> Result<(), Error>;
    fn off(&mut self) -> Result<(), Error>;
}

pub struct WaterPumpImpl {
    pump: Pin,
}

impl WaterPumpImpl {
    pub fn new(pin: u64) -> Result<WaterPumpImpl, Error> {
        let water_pump = Pin::new(pin);
        water_pump.export()?;
        water_pump.set_direction(Direction::Out)?;
        water_pump.set_value(0)?;

        Ok(WaterPumpImpl { pump: water_pump })
    }
}

impl WaterPump for WaterPumpImpl {
    fn stop(&mut self) -> Result<(), Error> {
        self.off()
    }

    fn on(&mut self) -> Result<(), Error> {
        self.pump.set_value(1)
    }

    fn off(&mut self) -> Result<(), Error> {
        self.pump.set_value(0)
    }
}

pub struct WaterPumpMock {
    is_on: bool,
}

impl WaterPumpMock {
    pub fn new(_pin: u64) -> Result<WaterPumpMock, Error> {
        Ok(WaterPumpMock { is_on: false })
    }
}

impl WaterPump for WaterPumpMock {
    fn stop(&mut self) -> Result<(), Error> {
        println!("Pump is stopped");
        self.is_on = false;
        Ok(())
    }

    fn on(&mut self) -> Result<(), Error> {
        println!("Pump is on");
        self.is_on = true;
        Ok(())
    }

    fn off(&mut self) -> Result<(), Error> {
        println!("Pump is off");
        self.is_on = false;
        Ok(())
    }
}
