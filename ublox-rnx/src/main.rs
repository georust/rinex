//! Application to generate RINEX data in standard format
//! using a Ublox receiver.   
//! Homepage: <https://github.com/gwbres/rinex>
use clap::App;
use clap::load_yaml;
use std::str::FromStr;

use rinex::*;
use ublox::*;

mod device;

pub fn main () -> Result<(), Box<dyn std::error::Error>> {
	let yaml = load_yaml!("app.yml");
    let app = App::from_yaml(yaml);
	let matches = app.get_matches();

    // Port config
    let port = matches.value_of("port")
        .unwrap();
    let baud = matches.value_of("baud")
        .unwrap_or("9600");
    let baud = u32::from_str_radix(baud, 10)
        .unwrap();
    
    // open device
    let port = serialport::new(port, baud)
        .open()
        .expect(&format!("failed to open serial port \"{}\"", port));
    let device = device::Device::new(port);

    Ok(())
}
