use std::thread;
use std::time::Duration;

use mcp3008::Mcp3008;
use sysfs_gpio::{Direction, Pin};

const POLLING_TIME: Duration = Duration::from_secs(3);

const PUMP_ON: Duration = Duration::from_secs(2);
const PUMP_PIN: u64 = 4;

fn main() {
    if let Ok(mut mcp3008) = Mcp3008::new("/dev/spidev0.0") {
        let water_pump = Pin::new(PUMP_PIN);
        water_pump.export().unwrap();
        water_pump.set_direction(Direction::Out).unwrap();
        water_pump.set_value(0).unwrap();
        loop {
            println!("Channel 7 `{:?}`", mcp3008.read_adc(7));

            water_pump.set_value(1).unwrap();
            thread::sleep(PUMP_ON);
            water_pump.set_value(0).unwrap();

            thread::sleep(POLLING_TIME);
        }
    } else {
        println!("Unable to stablish contact with moisture sensor");
    }
}
