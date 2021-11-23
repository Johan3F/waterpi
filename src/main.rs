mod water_pump;

use std::thread;
use std::time::Duration;

use mcp3008::Mcp3008;
use sysfs_gpio::Error;

use water_pump::WaterPump;

const POLLING_TIME: Duration = Duration::from_secs(3);

const PUMP_ON: Duration = Duration::from_secs(2);
const PUMP_PIN: u64 = 4;

fn main() -> Result<(), Error> {
    let mut mcp3008 =
        Mcp3008::new("/dev/spidev0.0").expect("Unable to establish connection with MCP3008");
    let pump = WaterPump::new(PUMP_PIN).expect("Unable to initialize water pump");

    loop {
        println!("Channel 7 `{:?}`", mcp3008.read_adc(7));

        pump.on().unwrap();
        thread::sleep(PUMP_ON);
        pump.off().unwrap();

        thread::sleep(POLLING_TIME);
    }
}
