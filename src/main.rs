mod controller;
mod metrics;
mod moisture_sensor;
mod water_pump;

use std::{cell::RefCell, rc::Rc, thread, time::Duration};

use crossbeam::channel::{select, unbounded, Receiver};
use ctrlc;
use failure::Error;
use log::info;
use prometheus::{self, default_registry, Encoder};
use warp::{Filter, Rejection, Reply};

// use controller::Controller;
use metrics::*;
use moisture_sensor::MoistureSensor;
// use water_pump::WaterPumpImpl;

const READ_CHANNEL: u8 = 7;
const SENSOR_POLLING_TIME: Duration = Duration::from_secs(1);

// const PUMP_PIN: u64 = 4;
// const WATERING_THRESHOLD: u16 = 500;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (quit_sender, quit_receiver) = unbounded();
    let quit_sender_copy = quit_sender.clone();
    ctrlc::set_handler(move || {
        quit_sender_copy
            .send(())
            .expect("Could not send quit signal...")
    })
    .expect("Error setting Ctrl-C handler");

    let sensor = MoistureSensor::new(READ_CHANNEL)?;
    // let pump = WaterPumpImpl::new(PUMP_PIN)?;
    // let pump = Rc::new(RefCell::new(pump));
    // let mut controller = Controller::new(WATERING_THRESHOLD, pump);

    tokio::task::spawn(web_server());

    let sensor = start_reading(sensor, SENSOR_POLLING_TIME);

    loop {
        select! {
            recv(sensor) -> received =>{
                match received {
                    Ok(value) => {
                        MOISTURE_LEVEL.set(value as f64);
                        // controller.new_reading(value)?;
                    }
                    Err(_) => {
                        // controller.stop();
                        break;
                    }
                }
            },
            recv(quit_receiver) -> _ => {
                println!("\nStopping system...");
                // controller.stop();
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
                    println!("Unable to get value this iteration. Waiting for next");
                }
            }
            Err(_) => {
                println!("Unable to get value this iteration. Waiting for next");
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
        eprintln!("could not encode custom metrics: {}", e);
    };
    let res = match String::from_utf8(buffer.clone()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("custom metrics could not be from_utf8'd: {}", e);
            String::default()
        }
    };
    buffer.clear();

    Ok(res)
}
