use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

use failure::Error;
use log::info;

use super::water_pump::WaterPump;

pub struct Controller {
    threshold: u16,
    pump: Rc<RefCell<dyn WaterPump>>,
    last_water_time: Option<Instant>,
}

impl Controller {
    pub fn new(threshold: u16, pump: Rc<RefCell<dyn WaterPump>>) -> Controller {
        Controller {
            threshold,
            pump,
            last_water_time: None,
        }
    }

    pub fn new_reading(&mut self, reading: u16) -> Result<(), Error> {
        info!("New reading: {}", reading);
        if reading > self.threshold {
            self.threshold_breached();
        } else {
            self.below_threshold();
        }
        Ok(())
    }
    pub fn stop(&mut self) {
        let _ = self.pump.borrow_mut().stop();
    }

    fn threshold_breached(&mut self) {
        let _ = self.pump.borrow_mut().on();
        self.last_water_time = Some(Instant::now());
    }

    fn below_threshold(&mut self) {
        let _ = self.pump.borrow_mut().off();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::water_pump::WaterPumpMock;

    #[test]
    fn test_controller() {
        let mock_pump = Rc::new(RefCell::new(WaterPumpMock::new()));

        let mut controller = Controller::new(600, mock_pump.clone());

        let result = controller.new_reading(500);
        assert_eq!(result.is_ok(), true);
        assert_eq!(mock_pump.borrow().is_on, false);

        let result = controller.new_reading(601);
        assert_eq!(result.is_ok(), true);
        assert_eq!(mock_pump.borrow().is_on, true);
    }
}
