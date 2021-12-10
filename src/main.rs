mod settings;
mod waterpi;

use std::{cell::RefCell, rc::Rc, thread, time::Duration};

use anyhow::{Context, Result};
use crossbeam::channel::{unbounded, Receiver, Select};
use ctrlc;
use log::{error, info, warn};
use prometheus::{self, default_registry, Encoder};
use warp::{Filter, Rejection, Reply};

use waterpi::controller::Controller;
use waterpi::metrics::*;
use waterpi::moisture_sensor::MoistureSensor;
use waterpi::water_pump::WaterPumpImpl;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder().format_module_path(false).init();
    let config = settings::get_settings()?;

    let (quit_sender, quit_receiver) = unbounded();
    let quit_sender_copy = quit_sender.clone();
    ctrlc::set_handler(move || {
        quit_sender_copy
            .send(())
            .expect("Could not send quit signal...");
    })
    .with_context(|| format!("Error setting Ctrl-C handler"))?;
    tokio::task::spawn(web_server());

    let mut sel = Select::new();
    sel.recv(&quit_receiver);
    let mut controllers = vec![];
    let mut sensors = vec![];
    for sensor_pump in &config.sensors_pumps {
        println!("{:?}", sensor_pump);
        let sensor = MoistureSensor::new(sensor_pump.sensor_channel)?;
        let pump = WaterPumpImpl::new(sensor_pump.pump_pin, sensor_pump.dry_run)?;
        let pump = Rc::new(RefCell::new(pump));
        let controller = Controller::new(
            sensor_pump.watering_threshold,
            Duration::from_secs(sensor_pump.watering_throttle * 60 * 60),
            pump,
        );
        let sensor = start_reading(
            sensor,
            Duration::from_secs(sensor_pump.sensor_polling_time_seconds),
        );

        controllers.push(controller);
        sensors.push(sensor);
    }

    for sensor in &sensors {
        sel.recv(&sensor);
    }

    loop {
        let operation = sel.select();
        if operation.index() == 0 {
            info!("\nStopping system...");
            break;
        }

        let index = operation.index() - 1;
        let sensor = &sensors[index];
        let controller = &mut controllers[index];

        match operation.recv(&sensor) {
            Ok(value) => {
                MOISTURE_LEVEL
                    .with_label_values(&[&format!(
                        "{}",
                        config.sensors_pumps[index].sensor_channel
                    )])
                    .set(value as f64);
                controller.new_reading(value)?;
            }
            Err(_) => {
                break;
            }
        }
    }

    for mut controller in controllers {
        controller.stop();
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
