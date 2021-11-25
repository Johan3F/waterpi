use failure::Error;

use super::water_pump::WaterPump;

pub struct Controller<T: WaterPump> {
    threshold: u16,
    pump: T,
}

impl<T: WaterPump> Controller<T> {
    pub fn new(threshold: u16, pump: T) -> Controller<T> {
        Controller { threshold, pump }
    }

    pub fn new_reading(&mut self, reading: u16) -> Result<(), Error> {
        println!("New reading: {}", reading);
        if reading > self.threshold {
            let _ = self.pump.on();
        } else {
            let _ = self.pump.off();
        }
        Ok(())
    }
    pub fn stop(&mut self) {
        let _ = self.pump.stop();
    }
}
