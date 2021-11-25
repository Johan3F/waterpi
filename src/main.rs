mod controller;
mod moisture_sensor;
mod water_pump;

use std::thread;
use std::time::Duration;

use crossbeam::channel::{select, unbounded, Receiver};
use ctrlc;
use failure::Error;

use controller::Controller;
use moisture_sensor::MoistureSensor;
use water_pump::WaterPumpImpl;

const READ_CHANNEL: u8 = 7;
const SENSOR_POLLING_TIME: Duration = Duration::from_secs(1);

const PUMP_PIN: u64 = 4;
const WATERING_THRESHOLD: u16 = 500;

fn main() -> Result<(), Error> {
    let (quit_sender, quit_receiver) = unbounded();
    let quit_sender_copy = quit_sender.clone();
    ctrlc::set_handler(move || {
        quit_sender_copy
            .send(())
            .expect("Could not send quit signal...")
    })
    .expect("Error setting Ctrl-C handler");

    let sensor = MoistureSensor::new(READ_CHANNEL)?;
    let pump = WaterPumpImpl::new(PUMP_PIN)?;
    let mut controller = Controller::new(WATERING_THRESHOLD, pump);

    let sensor = start_reading(sensor, SENSOR_POLLING_TIME);

    loop {
        select! {
            recv(sensor) -> received =>{
                match received {
                    Ok(value) => {
                        controller.new_reading(value)?;
                    }
                    Err(_) => {
                        controller.stop();
                        break;
                    }
                }
            },
            recv(quit_receiver) -> _ => {
                println!("\nStopping system...");
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
