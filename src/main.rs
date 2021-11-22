use std::thread;
use std::time::Duration;

use mcp3008::Mcp3008;

fn main() {
    if let Ok(mut mcp3008) = Mcp3008::new("/dev/spidev0.0") {
        loop {
            println!(
                "Channel 1 `{:?}` | Channel 7 `{:?}`",
                mcp3008.read_adc(1),
                mcp3008.read_adc(7)
            );
            thread::sleep(Duration::from_secs(1))
        }
    }
}
