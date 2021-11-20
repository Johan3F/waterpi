use std::thread::sleep;
use std::time::Duration;

use sysfs_gpio::{Direction, Pin};

const MCP: u64 = 12;

fn main() {
    let mcp = Pin::new(MCP);
    mcp.with_exported(|| {
        mcp.set_direction(Direction::In)?;
        loop {
            let val = mcp.get_value()?;
            println!("Pin State: {}", if val == 0 { "Low" } else { "High" });
            sleep(Duration::from_millis(100));
        }
    })
    .unwrap();
}
