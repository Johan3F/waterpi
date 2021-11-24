mod water_pump;

use std::thread;
use std::time::Duration;

use crossbeam::channel::{select, unbounded, Receiver};
use ctrlc;
use mcp3008::Mcp3008;
use sysfs_gpio::Error;

use water_pump::WaterPump;

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

    let mcp3008 =
        Mcp3008::new("/dev/spidev0.0").expect("Unable to establish connection with MCP3008");
    let pump = WaterPump::new(PUMP_PIN).expect("Unable to initialize water pump");

    let sensor = start_reading(
        mcp3008,
        quit_receiver.clone(),
        SENSOR_POLLING_TIME,
        READ_CHANNEL,
    );

    loop {
        let received = sensor.recv();
        match received {
            Ok(value) => {
                println!("Sensor readings: {}", value);
                if value > WATERING_THRESHOLD {
                    let _ = pump.on();
                } else {
                    let _ = pump.off();
                }
            }
            Err(_) => {
                let _ = pump.stop();
                let _ = quit_sender.send(());
                break;
            }
        }
    }

    Ok(())
}

fn start_reading(
    mut mcp3008: Mcp3008,
    quit_channel: Receiver<()>,
    polling_time: Duration,
    channel: u8,
) -> Receiver<u16> {
    let (internal_sender, internal_receiver) = unbounded();
    std::thread::spawn(move || loop {
        match mcp3008.read_adc(channel) {
            Ok(read_value) => {
                if let Err(_) = internal_sender.send(read_value) {
                    println!("Unable to get value this iteration. Waiting for next");
                }
            }
            Err(_) => {
                println!("Unable to get value this iteration. Waiting for next");
            }
        };
        thread::sleep(polling_time);
    });

    let (outside_sender, outside_receiver) = unbounded();
    std::thread::spawn(move || loop {
        select! {
            recv(internal_receiver) -> received => {
                match received {
                    Ok(value) => {
                        if let Err(_) = outside_sender.send(value) {
                            break;
                        }
                    }
                    Err(_) => break,
                };
            },
            recv(quit_channel) -> _ => {
                println!("\nEnding sensor polling");
                break;
            },
        }
    });

    outside_receiver
}
