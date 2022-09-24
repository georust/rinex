//! Application to generate RINEX data in standard format
//! using a Ublox receiver.   
//! Homepage: <https://github.com/gwbres/rinex>
use clap::App;
use clap::load_yaml;
//use std::str::FromStr;

use rinex::*;
use rinex::sv::Sv;
use rinex::epoch::Epoch;
use rinex::observation::record::ObservationData;

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
    device
        .wait_for_ack::<CfgPrtUart>()
        .unwrap();
    
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
    device
        .wait_for_ack::<CfgPrtUart>()
        .unwrap();
    
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
    device
        .wait_for_ack::<CfgPrtUart>()
        .unwrap();

    ///////////////////////
    // Observation opmode
    ///////////////////////
    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<NavSat>([0, 1, 0, 0, 0, 0])
                .into_packet_bytes())
        .unwrap();
    device
        .wait_for_ack::<CfgMsgAllPorts>()
        .unwrap();
    
    ///////////////////////
    // Navigation opmode
    ///////////////////////
    // Enable GPS Ephemeris + GPS Iono
    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<MgaGpsEph>([0, 1, 0, 0, 0, 0])
                .into_packet_bytes(),
        )
    .unwrap();
    device
        .wait_for_ack::<CfgMsgAllPorts>()
        .unwrap();
    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<MgaGpsEph>([0, 1, 0, 0, 0, 0])
                .into_packet_bytes(),
        )
    .unwrap();
    device
        .wait_for_ack::<CfgMsgAllPorts>()
        .unwrap();

    // Create header section
    let header = header::Header::basic_obs(); 

    //TODO header customization
    
    let mut epoch = Epoch::default(); // current epoch 

    loop { // main loop
        let _ = device.
            update(|packet| {
                match packet {
                    PacketRef::NavSat(pkt) => {
                        for sv in pkt.svs() {
                            let gnss_id = sv.gnss_id();
                            let sv_id = sv.sv_id();
                            let elev = sv.elev();
                            let azim = sv.azim();
                            let pr_res = sv.pr_res();
                            let flags = sv.flags();
                            //if flags.sv_used() {
                            //}
                            //flags.health();
                            //flags.quality_ind();
                            //flags.differential_correction_available();
                            //flags.ephemeris_available();
                        }
                    }
                    PacketRef::NavEoe(pkt) => { // End of epoch notification
                        let itow = pkt.itow();
                        // ==> push into file
                    }
                    _ => {},
                }
            });
        
    }
    //Ok(())
}
