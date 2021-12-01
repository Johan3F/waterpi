use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SensorPumpPair {
    pub sensor_channel: u8,
    pub sensor_polling_time_seconds: u64,
    pub watering_threshold: u16,
    pub watering_throttle: u64,
    pub pump_pin: u64,
    pub dry_run: bool,
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub sensors_pumps: Vec<SensorPumpPair>,
}

pub fn get_settings() -> Result<Configuration> {
    let mut settings = config::Config::default();
    settings.merge(config::File::with_name("config")).unwrap();

    settings
        .try_into::<Configuration>()
        .with_context(|| format!("Failed to read configuration"))
}
