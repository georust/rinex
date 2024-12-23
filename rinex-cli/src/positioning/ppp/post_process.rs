use std::{
    collections::{BTreeMap, HashMap},
    io::Write,
};

use crate::cli::Context;

use clap::ArgMatches;
use gnss_rtk::prelude::{Epoch, PVTSolution};
use thiserror::Error;

#[cfg(feature = "gpx")]
extern crate gpx;

#[cfg(feature = "gpx")]
use gpx::{errors::GpxError, Gpx, GpxVersion, Waypoint};

#[cfg(feature = "kml")]
use kml::{
    types::AltitudeMode, types::Coord as KmlCoord, types::Geometry as KmlGeometry,
    types::KmlDocument, types::Placemark, types::Point as KmlPoint, Kml, KmlVersion, KmlWriter,
};

extern crate geo_types;
use geo_types::Point as GeoPoint;

#[derive(Debug, Error)]
pub enum Error {
    #[error("std::io error")]
    IOError(#[from] std::io::Error),
    #[cfg(feature = "gpx")]
    #[error("failed to generate gpx track")]
    GpxError(#[from] GpxError),
    #[cfg(feature = "kml")]
    #[error("failed to generate kml track")]
    KmlError(#[from] kml::Error),
}

pub fn post_process(
    ctx: &Context,
    solutions: &BTreeMap<Epoch, PVTSolution>,
    matches: &ArgMatches,
) -> Result<(), Error> {
    /*
     * Generate txt, GPX, KML..
     */
    let mut fd = ctx.workspace.create_file("Solutions.csv");

    #[cfg(feature = "gpx")]
    let mut gpx_track = gpx::Track::default();

    #[cfg(feature = "kml")]
    let mut kml_track = Vec::<Kml>::new();

    writeln!(
        fd,
        "Epoch, x_ecef [m], y_ecef [m], z_ecef [m], vel_x [m/s], vel_y [m/s], vel_z [m/s], altitude [m], hdop, vdop, rx_clock_offset, tdop"
    )?;

    for (epoch, solution) in solutions {
        let cartesian = solution.state.to_cartesian_pos_vel() * 1.0E3;
        let (x_m, y_m, z_m, vel_x_ms, vel_y_ms, vel_z_ms) = (
            cartesian[0],
            cartesian[1],
            cartesian[2],
            cartesian[3],
            cartesian[4],
            cartesian[5],
        );
        let (lat_deg, long_deg, alt_km) = solution.state.latlongalt().unwrap_or_else(|e| {
            panic!(
                "resolved invalid lat/long/altitude ({}) - check your input",
                e
            )
        });
        let alt_m = alt_km * 1.0E3;

        let (lat_rad, long_rad) = (lat_deg.to_radians(), long_deg.to_radians());
        let (hdop, vdop, tdop) = (
            solution.hdop(lat_rad, long_rad),
            solution.vdop(lat_rad, long_rad),
            solution.tdop,
        );
        writeln!(
            fd,
            "{:?}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}",
            epoch,
            x_m,
            y_m,
            z_m,
            vel_x_ms,
            vel_y_ms,
            vel_z_ms,
            alt_m,
            hdop,
            vdop,
            solution.dt.to_seconds(),
            tdop
        )?;

        #[cfg(feature = "gpx")]
        if matches.get_flag("gpx") {
            let mut segment = gpx::TrackSegment::new();
            let mut wp = Waypoint::new(GeoPoint::new(long_deg, lat_deg)); // longitude *then* latitude..
            wp.elevation = Some(alt_m);
            wp.speed = None; // TODO
            wp.time = None; // TODO Gpx::Time
            wp.name = Some(format!("{:?}", epoch));
            wp.hdop = Some(hdop);
            wp.vdop = Some(vdop);
            wp.sat = None; //TODO: nb of satellites
            wp.dgps_age = None; //TODO: Number of seconds since last DGPS update, from the element.
            wp.dgpsid = None; //TODO: ID of DGPS station used in differential correction, in the range [0, 1023].
            segment.points.push(wp);
            gpx_track.segments.push(segment);
        }

        #[cfg(feature = "kml")]
        if matches.get_flag("kml") {
            kml_track.push(Kml::Placemark(Placemark {
                name: Some(format!("{:?}", epoch)),
                description: Some(String::from("Rover")),
                geometry: {
                    Some(KmlGeometry::Point(KmlPoint {
                        coord: {
                            KmlCoord {
                                x: lat_deg,
                                y: long_deg,
                                z: Some(alt_m),
                            }
                        },
                        extrude: false,
                        altitude_mode: AltitudeMode::Absolute,
                        attrs: HashMap::new(),
                    }))
                },
                attrs: [(String::from("TDOP"), format!("{:.3E}", solution.tdop))]
                    .into_iter()
                    .collect(),
                children: vec![],
                style_url: None,
            }));
        }
    }

    #[cfg(feature = "gpx")]
    if matches.get_flag("gpx") {
        let prefix = ctx.name.clone();
        let fd = ctx.workspace.create_file(&format!("{}.gpx", prefix));

        let mut gpx = Gpx::default();
        gpx.version = GpxVersion::Gpx11;
        gpx_track.name = Some(prefix.clone());
        // gpx_track.number = Some(1);
        gpx.tracks.push(gpx_track);
        gpx::write(&gpx, fd)?;
    }
    #[cfg(not(feature = "gpx"))]
    if matches.get_flag("gpx") {
        panic!("--gpx option is not available: compile with gpx option");
    }

    #[cfg(feature = "kml")]
    if matches.get_flag("kml") {
        let prefix = ctx.name.clone();
        let mut fd = ctx.workspace.create_file(&format!("{}.kml", prefix));

        let kmldoc = KmlDocument {
            version: KmlVersion::V23,
            attrs: [(
                String::from("https://georust.org/"),
                env!("CARGO_PKG_VERSION").to_string(),
            )]
            .into_iter()
            .collect(),
            elements: {
                vec![Kml::Folder {
                    attrs: HashMap::new(),
                    elements: kml_track,
                }]
            },
        };
        let mut writer = KmlWriter::from_writer(&mut fd);
        writer.write(&Kml::KmlDocument(kmldoc))?;
    }
    #[cfg(not(feature = "kml"))]
    if matches.get_flag("kml") {
        panic!("--kml option is not available: compile with kml option");
    }

    Ok(())
}
