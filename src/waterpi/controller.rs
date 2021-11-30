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
    throttle: Duration,
    pump: Rc<RefCell<dyn WaterPump>>,
    last_water_time: Option<Instant>,
}

impl Controller {
    pub fn new(threshold: u16, throttle: Duration, pump: Rc<RefCell<dyn WaterPump>>) -> Controller {
        Controller {
            threshold,
            throttle,
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
        match self.last_water_time {
            None => {
                self.last_water_time = Some(Instant::now());
                let _ = self.pump.borrow_mut().on();
            }
            Some(last_water_time) => {
                if last_water_time.elapsed() >= self.throttle {
                    self.last_water_time = Some(Instant::now());
                    let _ = self.pump.borrow_mut().on();
                }
            }
        };
    }

    fn below_threshold(&mut self) {
        let _ = self.pump.borrow_mut().off();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::waterpi::water_pump::MockWaterPump;
    use std::thread::sleep;

    #[test]
    fn test_controller_throttle_not_expired() {
        let mock_pump = Rc::new(RefCell::new(MockWaterPump::new()));

        let mut controller = Controller::new(600, Duration::from_millis(200), mock_pump.clone());

        mock_pump
            .borrow_mut()
            .expect_off()
            .times(1)
            .returning(|| Ok(()));
        let result = controller.new_reading(500);
        assert_eq!(result.is_ok(), true);

        mock_pump
            .borrow_mut()
            .expect_on()
            .times(1)
            .returning(|| Ok(()));
        let result = controller.new_reading(601);
        assert_eq!(result.is_ok(), true);

        let _result = controller.new_reading(601);
        assert_eq!(result.is_ok(), true);

        mock_pump
            .borrow_mut()
            .expect_off()
            .times(1)
            .returning(|| Ok(()));
        let result = controller.new_reading(500);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_controller_throttle_expired() {
        let mock_pump = Rc::new(RefCell::new(MockWaterPump::new()));

        let mut controller = Controller::new(600, Duration::from_millis(100), mock_pump.clone());

        mock_pump
            .borrow_mut()
            .expect_on()
            .times(1)
            .returning(|| Ok(()));
        let result = controller.new_reading(601);
        assert_eq!(result.is_ok(), true);

        sleep(Duration::from_millis(100));

        mock_pump
            .borrow_mut()
            .expect_on()
            .times(1)
            .returning(|| Ok(()));
        let result = controller.new_reading(601);
        assert_eq!(result.is_ok(), true);

        mock_pump.borrow_mut().checkpoint();
    }
}
