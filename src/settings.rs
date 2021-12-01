use failure::Error;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct SensorPumpPair {
    sensor_channel: u8,
    sensor_polling_time_seconds: u64,
    watering_threshold: u16,
    watering_throttle: u64,
    pump_pin: u64,
    dry_run: bool,
}

#[derive(Debug, Deserialize)]
struct Configuration {
    sensor_pump: Vec<SensorPumpPair>,
}

pub fn get_settings() -> Result<(), Error> {
    let mut settings = config::Config::default();
    settings.merge(config::File::with_name("config")).unwrap();

    Ok(())
}
