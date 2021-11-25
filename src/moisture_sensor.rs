use failure::Error;
use mcp3008::Mcp3008;

pub struct MoistureSensor {
    sensor: Mcp3008,
    channel: u8,
}

impl MoistureSensor {
    pub fn new(channel: u8) -> Result<MoistureSensor, Error> {
        Ok(MoistureSensor {
            sensor: Mcp3008::new("/dev/spidev0.0")?,
            channel: channel,
        })
    }

    pub fn read(&mut self) -> Result<u16, Error> {
        match self.sensor.read_adc(self.channel) {
            Ok(value) => Ok(value),
            Err(error) => Err(Error::from(error)),
        }
    }
}
