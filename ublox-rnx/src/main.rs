//! Application to generate RINEX data in standard format
//! using a Ublox receiver.   
//! Homepage: <https://github.com/gwbres/rinex>
use clap::load_yaml;
use clap::App;
use std::str::FromStr;

use thiserror::Error;

use rinex::navigation::{IonMessage, KbModel, KbRegionCode};
use rinex::observation::ObservationData;
use rinex::prelude::*;
use rinex::sv;

extern crate ublox;
use ublox::{
    CfgMsgAllPorts, CfgMsgAllPortsBuilder, CfgPrtUart, CfgPrtUartBuilder, DataBits, InProtoMask,
    OutProtoMask, PacketRef, Parity, StopBits, UartMode, UartPortId,
};
use ublox::{GpsFix, RecStatFlags};
use ublox::{NavSat, NavTimeUtcFlags};
use ublox::{NavStatusFlags, NavStatusFlags2};

use log::{debug, error, info, trace, warn};

mod device;

#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("unknown constellation #{0}")]
    UnknownConstellationId(u8),
}

fn identify_constellation(id: u8) -> Result<Constellation, Error> {
    match id {
        0 => Ok(Constellation::GPS),
        1 => Ok(Constellation::Galileo),
        2 => Ok(Constellation::Glonass),
        3 => Ok(Constellation::BeiDou),
        _ => Err(Error::UnknownConstellationId(id)),
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let yaml = load_yaml!("app.yml");
    let app = App::from_yaml(yaml);
    let matches = app.get_matches();

    // Port config
    let port = matches.value_of("port").unwrap();
    let baud = matches.value_of("baud").unwrap_or("9600");
    let baud = u32::from_str_radix(baud, 10).unwrap();

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
    device.wait_for_ack::<CfgPrtUart>().unwrap();

    /*
     * HEADER <=> Configuration
     */
    //CfgNav5 : model, dynamics..
    //CfgNav5X : min_svs, aiding, wkn, ppp..
    //AidIni

    /* NEED UBX CRATE UPDATE!!
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
    device.wait_for_ack::<CfgPrtUart>().unwrap();
    */

    /* NEED UBX CRATE UPDATE!!
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
    device.wait_for_ack::<CfgPrtUart>().unwrap();
    */

    ///////////////////////
    // Observation opmode
    ///////////////////////
    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<NavSat>([0, 1, 0, 0, 0, 0]).into_packet_bytes(),
        )
        .unwrap();
    device.wait_for_ack::<CfgMsgAllPorts>().unwrap();

    ///////////////////////
    // Navigation opmode
    ///////////////////////
    // Enable GPS Ephemeris + GPS Iono

    /* NEED UBX Crate update!!
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
    */

    // Create header section
    let mut nav_header = Header::basic_nav();
    let mut obs_header = Header::basic_obs();
    // let mut clk_header = Header::basic_clk();

    //TODO header CLI customization

    // current work structures
    let mut itow = 0_u32;
    let mut epoch = Epoch::default();
    let mut observable = Observable::default();
    let mut obs_data = ObservationData::default();
    let mut uptime = Duration::default();

    let mut fix_type = GpsFix::NoFix; // current fix status
    let mut fix_flags = NavStatusFlags::empty(); // current fix flag
    let mut nav_status = NavStatusFlags2::Inactive;

    loop {
        // main loop
        let _ = device.update(|packet| {
            match packet {
                /*
                 * Configuration frames:
                 * should be depiceted by HEADER section
                 */
                //PacketRef::CfgRate(pkt) => {
                //    //TODO EPOCH INTERVAL
                //    let gps_rate = pkt.measure_rate_ms();
                //    //TODO EPOCH INTERVAL
                //    let nav_rate = pkt.nav_rate();
                //    //TODO reference time
                //    let time = pkt.time_ref();
                //},
                PacketRef::CfgNav5(pkt) => {
                    // Dynamic model
                    let dyn_model = pkt.dyn_model();
                },
                PacketRef::RxmRawx(pkt) => {
                    let leap_s = pkt.leap_s();
                    if pkt.rec_stat().intersects(RecStatFlags::CLK_RESET) {
                        // notify reset + lli
                    }
                },
                PacketRef::MonHw(_pkt) => {
                    //let jamming = pkt.jam_ind(); //TODO
                    //antenna problem:
                    // pkt.a_status();
                    // pkt.a_power();
                },
                PacketRef::MonGnss(_pkt) => {
                    //pkt.supported(); // GNSS
                    //pkt.default(); // GNSS
                    //pkt.enabled(); //GNSS
                },
                PacketRef::MonVer(pkt) => {
                    //UBX revision
                    pkt.software_version();
                    pkt.hardware_version();
                },
                /*
                 * NAVIGATION
                 */
                PacketRef::NavSat(pkt) => {
                    for sv in pkt.svs() {
                        let gnss = identify_constellation(sv.gnss_id());
                        if gnss.is_ok() {
                            let elev = sv.elev();
                            let azim = sv.azim();
                            let pr_res = sv.pr_res();
                            let flags = sv.flags();

                            let sv = Sv {
                                constellation: gnss.unwrap(),
                                prn: sv.sv_id(),
                            };

                            // flags.sv_used()
                            //flags.health();
                            //flags.quality_ind();
                            //flags.differential_correction_available();
                            //flags.ephemeris_available();
                        }
                    }
                },
                PacketRef::NavTimeUTC(pkt) => {
                    if pkt.valid().intersects(NavTimeUtcFlags::VALID_UTC) {
                        // leap seconds already known
                        let e = Epoch::maybe_from_gregorian(
                            pkt.year().into(),
                            pkt.month().into(),
                            pkt.day().into(),
                            pkt.hour().into(),
                            pkt.min().into(),
                            pkt.sec().into(),
                            pkt.nanos() as u32,
                            TimeScale::UTC,
                        );
                        if e.is_ok() {
                            epoch = e.unwrap();
                        }
                    }
                },
                PacketRef::NavStatus(pkt) => {
                    itow = pkt.itow();
                    fix_type = pkt.fix_type();
                    fix_flags = pkt.flags();
                    nav_status = pkt.flags2();
                    uptime = Duration::from_milliseconds(pkt.uptime_ms() as f64);
                    trace!("uptime: {}", uptime);
                },
                PacketRef::NavEoe(pkt) => {
                    itow = pkt.itow();
                },
                /*
                 * NAVIGATION : EPHEMERIS
                 */
                PacketRef::MgaGpsEph(pkt) => {
                    let sv = sv!(&format!("G{}", pkt.sv_id()));
                    //nav_record.insert(epoch, sv);
                },
                PacketRef::MgaGloEph(pkt) => {
                    let sv = sv!(&format!("R{}", pkt.sv_id()));
                    //nav_record.insert(epoch, sv);
                },
                /*
                 * NAVIGATION: IONOSPHERIC MODELS
                 */
                PacketRef::MgaGpsIono(pkt) => {
                    let kbmodel = KbModel {
                        alpha: (pkt.alpha0(), pkt.alpha1(), pkt.alpha2(), pkt.alpha3()),
                        beta: (pkt.beta0(), pkt.beta1(), pkt.beta2(), pkt.beta3()),
                        region: KbRegionCode::default(), // TODO,
                    };
                    let iono = IonMessage::KlobucharModel(kbmodel);
                },
                /*
                 * OBSERVATION: Receiver Clock
                 */
                PacketRef::NavClock(pkt) => {
                    let bias = pkt.clk_b();
                    let drift = pkt.clk_d();
                    // pkt.t_acc(); // phase accuracy
                    // pkt.f_acc(); // frequency accuracy
                },
                /*
                 * Errors, Warnings
                 */
                PacketRef::InfTest(pkt) => {
                    if let Some(msg) = pkt.message() {
                        trace!("{}", msg);
                    }
                },
                PacketRef::InfDebug(pkt) => {
                    if let Some(msg) = pkt.message() {
                        debug!("{}", msg);
                    }
                },
                PacketRef::InfNotice(pkt) => {
                    if let Some(msg) = pkt.message() {
                        info!("{}", msg);
                    }
                },
                PacketRef::InfError(pkt) => {
                    if let Some(msg) = pkt.message() {
                        error!("{}", msg);
                    }
                },
                PacketRef::InfWarning(pkt) => {
                    if let Some(msg) = pkt.message() {
                        warn!("{}", msg);
                    }
                },
                _ => {},
            }
        });
    }
    //Ok(())
}
