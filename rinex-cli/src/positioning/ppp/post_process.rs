use std::{
    collections::{BTreeMap, HashMap},
    io::Write,
};

use crate::cli::Context;

use clap::ArgMatches;
use itertools::Itertools;
use rtk::prelude::{Carrier, Config, Epoch, Method, PVTSolution, SV};
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

use map_3d::{deg2rad, ecef2geodetic, rad2deg, Ellipsoid};

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
        "Epoch, x_ecef, y_ecef, z_ecef, speed_x, speed_y, speed_z, hdop, vdop, rx_clock_offset, tdop"
    )?;

    for (epoch, solution) in solutions {
        let (lat_rad, lon_rad, alt) = ecef2geodetic(
            solution.position.x,
            solution.position.y,
            solution.position.z,
            Ellipsoid::WGS84,
        );
        let (lat_ddeg, lon_ddeg) = (rad2deg(lat_rad), rad2deg(lon_rad));
        let (hdop, vdop, tdop) = (
            solution.hdop(lat_rad, lon_rad),
            solution.vdop(lat_rad, lon_rad),
            solution.tdop,
        );
        writeln!(
            fd,
            "{:?}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}",
            epoch,
            solution.position.x,
            solution.position.y,
            solution.position.z,
            solution.velocity.x,
            solution.velocity.y,
            solution.velocity.z,
            hdop,
            vdop,
            solution.dt.to_seconds(),
            tdop
        )?;

        #[cfg(feature = "gpx")]
        if matches.get_flag("gpx") {
            let mut segment = gpx::TrackSegment::new();
            let mut wp = Waypoint::new(GeoPoint::new(lon_ddeg, lat_ddeg)); // Yes, longitude *then* latitude
            wp.elevation = Some(alt);
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
                description: Some(String::from("\"Receiver Location\"")),
                geometry: {
                    Some(KmlGeometry::Point(KmlPoint {
                        coord: {
                            KmlCoord {
                                x: lat_ddeg,
                                y: lon_ddeg,
                                z: Some(alt),
                            }
                        },
                        extrude: false,
                        altitude_mode: AltitudeMode::Absolute,
                        attrs: HashMap::new(),
                    }))
                },
                attrs: [(String::from("TDOP"), format!("{:.6E}", solution.tdop))]
                    .into_iter()
                    .collect(),
                children: vec![],
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
