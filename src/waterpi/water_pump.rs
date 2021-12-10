use anyhow::{Context, Result};
use sysfs_gpio::{Direction, Pin};

use crate::waterpi::metrics::*;

#[cfg(test)]
use mockall::{automock, predicate::*};

#[cfg_attr(test, automock)]
pub trait WaterPump: Send {
    fn stop(&mut self) -> Result<()>;
    fn on(&mut self) -> Result<()>;
    fn off(&mut self) -> Result<()>;
}

pub struct WaterPumpImpl {
    pump: Pin,
    dry_run: bool,
}

impl WaterPumpImpl {
    pub fn new(pin: u64, dry_run: bool) -> Result<WaterPumpImpl> {
        let water_pump = Pin::new(pin);
        water_pump.export()?;
        water_pump.set_direction(Direction::Out)?;
        water_pump.set_value(0)?;
        PUMP_ON
            .with_label_values(&[&water_pump.get_pin().to_string()])
            .set(0.0);

        Ok(WaterPumpImpl {
            pump: water_pump,
            dry_run,
        })
    }
}

impl WaterPump for WaterPumpImpl {
    fn stop(&mut self) -> Result<()> {
        if self.dry_run {
            return Ok(());
        }
        self.off()
    }

    fn on(&mut self) -> Result<()> {
        PUMP_ON
            .with_label_values(&[&self.pump.get_pin().to_string()])
            .set(1.0);
        if self.dry_run {
            return Ok(());
        }
        self.pump
            .set_value(1)
            .with_context(|| "Unable to turn the pump on".to_owned())
    }

    fn off(&mut self) -> Result<()> {
        PUMP_ON
            .with_label_values(&[&self.pump.get_pin().to_string()])
            .set(0.0);
        if self.dry_run {
            return Ok(());
        }
        self.pump
            .set_value(0)
            .with_context(|| "Unable to turn the pump off".to_owned())
    }
}
