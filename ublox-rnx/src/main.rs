//! Application to generate RINEX data in standard format
//! using a Ublox receiver.   
//! Homepage: <https://github.com/gwbres/rinex>
use clap::App;
use clap::load_yaml;
use std::str::FromStr;

use rinex::*;

extern crate ublox;
use ublox::*;
use ublox::{CfgPrtUart, UartPortId};

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

    // Parameters
    let obs = matches.is_present("obs");
    let nav = matches.is_present("nav");
    //TODO: currently only supports GPS
    
    // open device
    let port = serialport::new(port, baud)
        .open()
        .expect(&format!("failed to open serial port \"{}\"", port));
    let mut device = device::Device::new(port);
        
    // Enable UBX protocol on all ports
    // so User can connect to all of them
    device.write_all(
        &CfgPrtUartBuilder {
            portid: UartPortId::Uart1,
            reserved0: 0,
            tx_ready: 0,
            mode: UartMode::new(DataBits::Eight, Parity::None, StopBits::One),
            baud_rate: baud, 
            in_proto_mask: InProtoMask::all(),
            out_proto_mask: OutProtoMask::UBLOX, 
            flags: 0,
            reserved5: 0,
        }
        .into_packet_bytes(),
    )?;
    device.wait_for_ack::<CfgPrtUart>()?;
    
    device.write_all(
        &CfgPrtUartBuilder {
            portid: UartPortId::Uart2,
            reserved0: 0,
            tx_ready: 0,
            mode: UartMode::new(DataBits::Eight, Parity::None, StopBits::One),
            baud_rate: baud, 
            in_proto_mask: InProtoMask::all(),
            out_proto_mask: OutProtoMask::UBLOX, 
            flags: 0,
            reserved5: 0,
        }
        .into_packet_bytes(),
    )?;
    device.wait_for_ack::<CfgPrtUart>()?;
    
    device.write_all(
        &CfgPrtUartBuilder {
            portid: UartPortId::Usb,
            reserved0: 0,
            tx_ready: 0,
            mode: UartMode::new(DataBits::Eight, Parity::None, StopBits::One),
            baud_rate: baud, 
            in_proto_mask: InProtoMask::all(),
            out_proto_mask: OutProtoMask::UBLOX, 
            flags: 0,
            reserved5: 0,
        }
        .into_packet_bytes(),
    )?;
    device.wait_for_ack::<CfgPrtUart>()?;

    // Enable GPS Ephemeris + GPS Iono
    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<MgaGpsEph>([0, 1, 0, 0, 0, 0])
                .into_packet_bytes(),
        )
    .unwrap();
    device.wait_for_ack::<CfgMsgAllPorts>().unwrap();
    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<MgaGpsEph>([0, 1, 0, 0, 0, 0])
                .into_packet_bytes(),
        )
    .unwrap();
    device.wait_for_ack::<CfgMsgAllPorts>().unwrap();

    // Create OBS file
    let mut obs = Rinex::default();
    let mut header = header::Header::basic_obs(); 

    loop {
        
    }

    Ok(())
}
