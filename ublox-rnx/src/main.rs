//! Generate RINEX data from U-Blox GNSS receivers.
//! Homepage: <https://github.com/georust/rinex>

use thiserror::Error;

use rinex::{
    hardware::Receiver,
    navigation::{IonosphereModel, KbModel, KbRegionCode},
    observation::{ClockObservation, EpochFlag, LliFlags, SignalObservation},
    prelude::{Constellation, Duration, Epoch, Header, Observable, TimeScale, SV},
};

extern crate gnss_rs as gnss;
extern crate ublox;

use ublox::{
    CfgMsgAllPorts, CfgMsgAllPortsBuilder, CfgPrtUart, CfgPrtUartBuilder, DataBits, GpsFix,
    InProtoMask, NavSat, NavStatusFlags, NavStatusFlags2, NavTimeUtcFlags, OutProtoMask, PacketRef,
    Parity, RecStatFlags, StopBits, UartMode, UartPortId,
};

use log::{debug, error, info, trace, warn};

mod cli;
mod device;

use cli::Cli;

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
    // cli
    let cli = Cli::new();

    // Device configuration
    let port = cli.port();
    let baud_rate = match cli.baudrate() {
        Ok(b) => b,
        Err(e) => {
            error!("failed to parse baud_rate: {}", e);
            9600
        },
    };

    info!("connecting to {}, baud: {}", port, baud_rate);

    // open device
    let port = serialport::new(port.clone(), baud_rate)
        .open()
        .unwrap_or_else(|_| panic!("failed to open serial port \"{}\"", port));
    let mut device = device::Device::new(port);

    // Enable UBX protocol on all ports
    // so User can connect to all of them
    device.write_all(
        &CfgPrtUartBuilder {
            portid: UartPortId::Uart1,
            reserved0: 0,
            tx_ready: 0,
            mode: UartMode::new(DataBits::Eight, Parity::None, StopBits::One),
            baud_rate,
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
    let mut _nav_header = Header::basic_nav();
    let mut _obs_header = Header::basic_obs();
    // let mut clk_header = Header::basic_clk();

    //TODO header CLI customization

    // current work structures
    let mut itow = 0_u32;
    let mut epoch = Epoch::default();
    let mut epoch_flag = EpochFlag::default();

    // observation
    let mut t = Epoch::default();
    let mut rcvr = Receiver::default();
    let mut lli: Option<LliFlags> = None;
    let mut signal = SignalObservation::default();
    let mut clock = ClockObservation::default();

    let mut _observable = Observable::default();

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
                    let _dyn_model = pkt.dyn_model();
                },
                PacketRef::RxmRawx(pkt) => {
                    let _leap_s = pkt.leap_s();
                    if pkt.rec_stat().intersects(RecStatFlags::CLK_RESET) {
                        // notify reset event
                        if let Some(ref mut lli) = lli {
                            *lli |= LliFlags::LOCK_LOSS;
                        } else {
                            lli = Some(LliFlags::LOCK_LOSS);
                        }
                        epoch_flag = EpochFlag::CycleSlip;
                    }
                    signal.lli = lli;
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
                    let sw_version = pkt.software_version();
                    let hw_version = pkt.hardware_version();
                },
                /*
                 * NAVIGATION
                 */
                PacketRef::NavSat(pkt) => {
                    for sv in pkt.svs() {
                        let gnss = identify_constellation(sv.gnss_id());
                        if gnss.is_ok() {
                            let _elev = sv.elev();
                            let _azim = sv.azim();
                            let _pr_res = sv.pr_res();
                            let _flags = sv.flags();

                            let _sv = SV {
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
                            pkt.month(),
                            pkt.day(),
                            pkt.hour(),
                            pkt.min(),
                            pkt.sec(),
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
                    // reset Epoch
                    lli = None;
                    signal = Default::default();
                    epoch_flag = EpochFlag::default();
                },
                /*
                 * NAVIGATION : EPHEMERIS
                 */
                PacketRef::MgaGpsEph(_pkt) => {
                    //let _sv = sv!(&format!("G{}", pkt.sv_id()));
                    //nav_record.insert(epoch, sv);
                },
                PacketRef::MgaGloEph(_pkt) => {
                    //let _sv = sv!(&format!("R{}", pkt.sv_id()));
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
                    let _iono = IonosphereModel::Klobuchar(kbmodel);
                },
                /*
                 * OBSERVATION: Receiver Clock
                 */
                PacketRef::NavClock(pkt) => {
                    //let bias = pkt.clk_b();
                    //let drift = pkt.clk_d();
                    //clock.with_offset_s(t, bias);
                    //clock.drift_s_s = drift.into();
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
