use crate::{fmt_rinex, ionex::Record, prelude::Header, FormattingError};

use itertools::Itertools;

use std::io::{BufWriter, Write};

pub fn format<W: Write>(
    w: &mut BufWriter<W>,
    record: &Record,
    header: &Header,
) -> Result<(), FormattingError> {
    const NUM_LONGITUDES_PER_LINE: usize = 16;

    let specs = header
        .ionex
        .as_ref()
        .ok_or(FormattingError::NoGridDefinition)?;

    // browse grid:
    // - for each altitude (km)
    // - - browse latitude (starting on northernmost.. to southernmost)
    // - - - browse longitude (starting on easternmost.. to westernmost)
    let grid = &specs.grid;

    let (mut altitude_low_km, altitude_high_km, altitude_spacing_km) =
        (grid.height.start, grid.height.end, grid.height.spacing);
    let (latitude_north_ddeg, latitude_south_ddeg, latitude_spacing_ddeg) = (
        grid.latitude.start,
        grid.latitude.end,
        grid.latitude.spacing,
    );
    let (longitude_east_ddeg, longitude_west_ddeg, longitude_spacing_ddeg) = (
        grid.longitude.start,
        grid.longitude.end,
        grid.longitude.spacing,
    );

    for t in record.keys().map(|k| k.epoch).unique().sorted() {

        // writeln!(
        //     w,
        //     "{}",
        //     fmt_rinex(
        //         &format!(
        //         "{:6}",
        //         map_count),
        //         "START OF TEC MAP"))?;

        // writeln!(
        //     w,
        //     fmt_rinex(
        //         format_utc_ionex(t),
        //         "EPOCH OF CURRENT MAP"))?;

        // while altitude_km < altitude_high_km {
        //     let mut latitude_ddeg = latitude_north_ddeg;
        //     while latitude_ddeg < latitude_south_ddeg {
        //         let mut longitude_item = 0u8;
        //         let mut longitude_ddeg = longitude_east_ddeg;
        //         writeln!(
        //             w,
        //                 fmt_rinex(
        //                     &format!(" {:3.1}{:3.1}{:3.1}  {:3.1} {:3.1}", latitude_ddeg,
        //                              longitude_east_ddeg, longitude_west_ddeg, longitude_spacing_ddeg,
        //                              altitude_km),
        //                              "LAT/LON1/LON2/DLON/H"))?;
        //         while longitude_ddeg < longitude_west_ddeg {

        //             let key = IonexMapsCoordinates::from_ddeg(
        //                 latitude_ddeg,
        //                 longitude_ddeg,
        //                 altitude_km);

        //             if let Some(tec) = rec.get(&key) {
        //                 write!(w, "{:5}", tec);
        //             } else {
        //                 // missing TEC
        //                 write!(w, "9999 ")?;
        //             }

        //             longitude_ddeg += longitude_spacing_ddeg;
        //             if longitude_item == 15 {
        //                 longitude_item = 0;
        //             } else {
        //                 longitude_item += 1;
        //             }
        //         }
        //         latitude_ddeg += latitude_spacing_ddeg;
        //     }
        //     altitude_km += altitude_spacing_km;
        // }
    }
    Ok(())
}
