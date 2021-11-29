mod controller;
mod metrics;
mod moisture_sensor;
mod water_pump;

use std::{cell::RefCell, rc::Rc, thread, time::Duration};

use crossbeam::channel::{select, unbounded, Receiver};
use ctrlc;
use dotenv::dotenv;
use failure::Error;
use log::{error, info, warn};
use prometheus::{self, default_registry, Encoder};
use structopt::StructOpt;
use warp::{Filter, Rejection, Reply};

use controller::Controller;
use metrics::*;
use moisture_sensor::MoistureSensor;
use water_pump::WaterPumpImpl;

#[derive(StructOpt)]
#[structopt(
    name = "WaterPi configurations",
    about = "Welcome to WaterPi! I'll read moisture level from a sensor connected with a MCP3008 and I react with a water pump"
)]
struct Configuration {
    #[structopt(
        long = "sensor_channel",
        env = "SENSOR_CHANNEL",
        help = "Channel in the MCP3008 that the sensor is connected"
    )]
    sensor_channel: u8,
    #[structopt(
        long = "sensor_polling_time",
        env = "SENSOR_POLLING_TIME",
        help = "Frequency to read sensor values. In seconds",
        default_value = "1"
    )]
    sensor_polling_time_seconds: u64,
    #[structopt(
        long = "pump_pin",
        env = "PUMP_PIN",
        help = "GPIO pin where the pump is connected"
    )]
    pump_pin: u64,
    #[structopt(
        long = "watering_threshold",
        env = "WATERING_THRESHOLD",
        help = "Threshold to consider a good time for watering the plant",
        default_value = "600"
    )]
    watering_threshold: u16,
    #[structopt(
        long = "watering_throttle",
        env = "WATERING_THROTTLE",
        help = "How often to allow watering in hours. To avoid watering too often",
        default_value = "1"
    )]
    watering_throttle: u64,
    #[structopt(
        long = "dry_run",
        env = "DRY_RUN",
        help = "Indicates if the pump should be enabled or not"
    )]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = dotenv();
    env_logger::builder().format_module_path(false).init();
    let config = Configuration::from_args();

    let (quit_sender, quit_receiver) = unbounded();
    let quit_sender_copy = quit_sender.clone();
    ctrlc::set_handler(move || {
        quit_sender_copy
            .send(())
            .expect("Could not send quit signal...")
    })
    .expect("Error setting Ctrl-C handler");

    let sensor = MoistureSensor::new(config.sensor_channel)?;
    let pump = WaterPumpImpl::new(config.pump_pin, config.dry_run)?;
    let pump = Rc::new(RefCell::new(pump));
    let mut controller = Controller::new(
        config.watering_threshold,
        Duration::from_secs(config.watering_throttle * 60 * 60),
        pump,
    );

    tokio::task::spawn(web_server());

    let sensor = start_reading(
        sensor,
        Duration::from_secs(config.sensor_polling_time_seconds),
    );

    loop {
        select! {
            recv(sensor) -> received =>{
                match received {
                    Ok(value) => {
                        MOISTURE_LEVEL.set(value as f64);
                        controller.new_reading(value)?;
                    }
                    Err(_) => {
                        controller.stop();
                        break;
                    }
                }
            },
            recv(quit_receiver) -> _ => {
                info!("\nStopping system...");
                controller.stop();
                break;
            }
        }
    }

    Ok(())
}

fn start_reading(mut sensor: MoistureSensor, polling_time: Duration) -> Receiver<u16> {
    let (sender, receiver) = unbounded();
    std::thread::spawn(move || loop {
        match sensor.read() {
            Ok(read_value) => {
                if let Err(_) = sender.send(read_value) {
                    warn!("Unable to get value this iteration. Waiting for next");
                }
            }
            Err(_) => {
                error!("Unable to get value this iteration. Waiting for next");
            }
        };
        thread::sleep(polling_time);
    });

    receiver
}

async fn web_server() {
    let metrics_route = warp::path!("metrics").and_then(metrics_handler);
    info!("Serving metrics on port '8080'. Endpoint '/metrics'");
    warp::serve(metrics_route).run(([0, 0, 0, 0], 8080)).await;
}

async fn metrics_handler() -> Result<impl Reply, Rejection> {
    let encoder = prometheus::TextEncoder::new();

    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&default_registry().gather(), &mut buffer) {
        error!("could not encode custom metrics: {}", e);
    };
    let res = match String::from_utf8(buffer.clone()) {
        Ok(v) => v,
        Err(e) => {
            error!("custom metrics could not be from_utf8'd: {}", e);
            String::default()
        }
    };
    buffer.clear();

    Ok(res)
}
