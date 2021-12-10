use prometheus::{self, Gauge, GaugeVec};

use lazy_static::lazy_static;
use prometheus::{register_gauge, register_gauge_vec};

lazy_static! {
    pub static ref MOISTURE_LEVEL: GaugeVec = register_gauge_vec!(
        "waterpi_moisture_level",
        "Vector of sensor and values",
        &["sensor"],
    )
    .unwrap();
    pub static ref PUMP_ON: Gauge =
        register_gauge!("waterpi_moisture_level", "1 is on, 0 is off").unwrap();
}
