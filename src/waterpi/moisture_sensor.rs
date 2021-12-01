use anyhow::{anyhow, Result};
use mcp3008::Mcp3008;

pub struct MoistureSensor {
    sensor: Mcp3008,
    channel: u8,
}

impl MoistureSensor {
    pub fn new(channel: u8) -> Result<MoistureSensor> {
        Ok(MoistureSensor {
            sensor: Mcp3008::new("/dev/spidev0.0")?,
            channel: channel,
        })
    }

    pub fn read(&mut self) -> Result<u16> {
        match self.sensor.read_adc(self.channel) {
            Ok(value) => Ok(value),
            Err(error) => Err(anyhow!(
                "Unable to read from sensor in channel {}: {}",
                self.channel,
                error
            )),
        }
    }
}
