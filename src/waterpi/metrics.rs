use prometheus::{self, GaugeVec};

use lazy_static::lazy_static;
use prometheus::register_gauge_vec;

lazy_static! {
    pub static ref MOISTURE_LEVELS: GaugeVec =
        register_gauge_vec!("waterpi_moisture_level", "sensor_values", &["sensor"]).unwrap();
    pub static ref PUMP_ON: GaugeVec =
        register_gauge_vec!("waterpi_pump_on", "1 is on, 0 is off", &["pump"]).unwrap();
}
