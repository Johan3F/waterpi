use prometheus::{self, Gauge};

use lazy_static::lazy_static;
use prometheus::{register_gauge, register_int_counter};

lazy_static! {
    pub static ref MOISTURE_LEVEL: Gauge =
        register_gauge!("waterpi.moisture_level", "moisture level").unwrap();
    pub static ref PUMP_ON: Gauge =
        register_gauge!("waterpi.moisture_level", "1 is on, 0 is off").unwrap();
}
