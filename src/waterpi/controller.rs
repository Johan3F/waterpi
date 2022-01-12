use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

use anyhow::Result;
use log::{debug, info};

use super::water_pump::WaterPump;
use crate::waterpi::metrics::MOISTURE_LEVELS;

pub struct Controller {
    name: String,
    threshold: u16,
    watering_throttle: Duration,
    watering_duration: Duration,
    pump: Rc<RefCell<dyn WaterPump>>,
    last_water_time: Option<Instant>,
}

impl Controller {
    pub fn new(
        name: String,
        threshold: u16,
        watering_throttle: Duration,
        watering_duration: Duration,
        pump: Rc<RefCell<dyn WaterPump>>,
    ) -> Controller {
        Controller {
            name,
            threshold,
            watering_throttle,
            watering_duration,
            pump,
            last_water_time: None,
        }
    }

    pub fn new_reading(&mut self, reading: u16) -> Result<()> {
        info!("New reading for {}: {}", self.name, reading);
        MOISTURE_LEVELS
            .with_label_values(&[&self.name])
            .set(reading as f64);
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
                debug!(
                    "[{}] threshold_breached -> last_water_time: {:?} \t watering_throttle: {:?}",
                    self.name,
                    last_water_time.elapsed(),
                    self.watering_throttle
                );
                if last_water_time.elapsed() >= self.watering_throttle {
                    self.last_water_time = Some(Instant::now());
                    let _ = self.pump.borrow_mut().on();
                } else if last_water_time.elapsed() >= self.watering_duration {
                    let _ = self.pump.borrow_mut().off();
                }
            }
        };
    }

    fn below_threshold(&mut self) {
        match self.last_water_time {
            None => (),
            Some(last_water_time) => {
                debug!(
                    "[{}] below_threshold -> last_water_time: {:?} /t watering_duration: {:?}",
                    self.name,
                    last_water_time.elapsed(),
                    self.watering_duration
                );
                if last_water_time.elapsed() >= self.watering_duration {
                    let _ = self.pump.borrow_mut().off();
                }
            }
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::waterpi::water_pump::MockWaterPump;
    use std::thread::sleep;

    #[test]
    fn test_controller_watering_throttle_not_expired() {
        let mock_pump = Rc::new(RefCell::new(MockWaterPump::new()));

        let mut controller = Controller::new(
            "test".to_owned(),
            600,
            Duration::from_millis(200),
            Duration::from_nanos(1),
            mock_pump.clone(),
        );

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

        std::thread::sleep(std::time::Duration::from_nanos(1));
        mock_pump
            .borrow_mut()
            .expect_off()
            .times(1)
            .returning(|| Ok(()));
        let result = controller.new_reading(500);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_controller_watering_throttle_expired() {
        let mock_pump = Rc::new(RefCell::new(MockWaterPump::new()));

        let mut controller = Controller::new(
            "test".to_owned(),
            600,
            Duration::from_millis(100),
            Duration::from_nanos(1),
            mock_pump.clone(),
        );

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

    #[test]
    fn test_controller_watering_duration_even_if_stil_too_dry() {
        let mock_pump = Rc::new(RefCell::new(MockWaterPump::new()));

        let mut controller = Controller::new(
            "test".to_owned(),
            600,
            Duration::from_millis(1000),
            Duration::from_millis(100),
            mock_pump.clone(),
        );

        mock_pump
            .borrow_mut()
            .expect_on()
            .times(1)
            .returning(|| Ok(()));
        mock_pump
            .borrow_mut()
            .expect_off()
            .times(0)
            .returning(|| Ok(()));
        let result = controller.new_reading(601);
        assert_eq!(result.is_ok(), true);

        let result = controller.new_reading(601);
        assert_eq!(result.is_ok(), true);

        mock_pump.borrow_mut().checkpoint();

        std::thread::sleep(Duration::from_millis(100));

        mock_pump
            .borrow_mut()
            .expect_on()
            .times(0)
            .returning(|| Ok(()));
        mock_pump
            .borrow_mut()
            .expect_off()
            .times(1)
            .returning(|| Ok(()));
        let result = controller.new_reading(601);
        assert_eq!(result.is_ok(), true);
        mock_pump.borrow_mut().checkpoint();
    }

    #[test]
    fn test_controller_watering_again_after_watering_throttle() {
        let mock_pump = Rc::new(RefCell::new(MockWaterPump::new()));

        let mut controller = Controller::new(
            "test".to_owned(),
            600,
            Duration::from_millis(1000),
            Duration::from_millis(100),
            mock_pump.clone(),
        );

        // Initial on
        mock_pump
            .borrow_mut()
            .expect_on()
            .times(1)
            .returning(|| Ok(()));
        mock_pump
            .borrow_mut()
            .expect_off()
            .times(0)
            .returning(|| Ok(()));
        let result = controller.new_reading(601);
        assert_eq!(result.is_ok(), true);
        let result = controller.new_reading(601);
        assert_eq!(result.is_ok(), true);
        mock_pump.borrow_mut().checkpoint();

        // Off after watering
        std::thread::sleep(Duration::from_millis(100));
        mock_pump
            .borrow_mut()
            .expect_on()
            .times(0)
            .returning(|| Ok(()));
        mock_pump
            .borrow_mut()
            .expect_off()
            .times(1)
            .returning(|| Ok(()));
        let result = controller.new_reading(601);
        assert_eq!(result.is_ok(), true);
        mock_pump.borrow_mut().checkpoint();

        // Back on after watering throttle
        std::thread::sleep(Duration::from_millis(1100));
        mock_pump
            .borrow_mut()
            .expect_on()
            .times(1)
            .returning(|| Ok(()));
        mock_pump
            .borrow_mut()
            .expect_off()
            .times(0)
            .returning(|| Ok(()));
        let result = controller.new_reading(601);
        assert_eq!(result.is_ok(), true);
        mock_pump.borrow_mut().checkpoint();

        // Off after watering
        std::thread::sleep(Duration::from_millis(100));
        mock_pump
            .borrow_mut()
            .expect_on()
            .times(0)
            .returning(|| Ok(()));
        mock_pump
            .borrow_mut()
            .expect_off()
            .times(1)
            .returning(|| Ok(()));
        let result = controller.new_reading(601);
        assert_eq!(result.is_ok(), true);
        mock_pump.borrow_mut().checkpoint();
    }
}
