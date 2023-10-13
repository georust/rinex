use crate::cli::Cli;
use hifitime::Epoch;
use rinex::prelude::RnxContext;
use rtk::prelude::SolverEstimate;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use thiserror::Error;

extern crate gpx;
use gpx::{errors::GpxError, Gpx, GpxVersion, Waypoint};

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
    #[error("failed to generate gpx track")]
    GpxError(#[from] GpxError),
    #[error("failed to generate kml track")]
    KmlError(#[from] kml::Error),
}

pub(crate) fn rtk_postproc(
    workspace: PathBuf,
    cli: &Cli,
    ctx: &RnxContext,
    results: HashMap<Epoch, SolverEstimate>,
) -> Result<(), Error> {
    let txtpath = workspace.join("rtk.txt");
    let txtfile = txtpath.to_string_lossy().to_string();
    let mut fd = File::create(&txtfile)?;

    let mut gpx_track = gpx::Track::default();
    let mut kml_track = Vec::<Kml>::new();

    let (x, y, z): (f64, f64, f64) = ctx
        .ground_position()
        .unwrap() // cannot fail
        .into();

    writeln!(
        fd,
        "Epoch, dx, dy, dz, x_ecef, y_ecef, z_ecef, hdop, vdop, rcvr_clock_bias, tdop"
    )?;

    for (epoch, estimate) in results {
        let (px, py, pz) = (x + estimate.dx, y + estimate.dy, z + estimate.dz);
        let (hdop, vdop, tdop) = (estimate.hdop, estimate.vdop, estimate.tdop);
        writeln!(
            fd,
            "{:?}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}",
            epoch, estimate.dx, estimate.dy, estimate.dz, px, py, pz, hdop, vdop, estimate.dt, tdop
        )?;
        if cli.rtk_gpx() {
            let mut segment = gpx::TrackSegment::new();
            let mut wp = Waypoint::new(GeoPoint::new(px, py));
            wp.elevation = Some(pz);
            wp.speed = None; // TODO ?
            wp.time = None; // TODO Gpx::Time
            wp.name = Some(format!("{:?}", epoch));
            wp.hdop = Some(hdop);
            wp.vdop = Some(vdop);
            wp.sat = None; //TODO: nb of contributing satellites
            wp.dgps_age = None; //TODO: Number of seconds since last DGPS update, from the element.
            wp.dgpsid = None; //TODO: ID of DGPS station used in differential correction, in the range [0, 1023].
            segment.points.push(wp);
            gpx_track.segments.push(segment);
        }
        if cli.rtk_kml() {
            kml_track.push(Kml::Placemark(Placemark {
                name: Some(format!("{:?}", epoch)),
                description: Some(String::from("\"Receiver Location\"")),
                geometry: {
                    Some(KmlGeometry::Point(KmlPoint {
                        coord: {
                            KmlCoord {
                                x: px,
                                y: py,
                                z: Some(pz),
                            }
                        },
                        extrude: false,
                        altitude_mode: AltitudeMode::Absolute,
                        attrs: HashMap::new(),
                    }))
                },
                attrs: [(String::from("TDOP"), format!("{:.6E}", estimate.tdop))]
                    .into_iter()
                    .collect(),
                children: vec![],
            }));
        }
    }
    info!("\"{}\" results generated", txtfile);
    if cli.rtk_gpx() {
        let gpxpath = workspace.join("rtk.gpx");
        let gpxfile = gpxpath.to_string_lossy().to_string();
        let mut fd = File::create(&gpxfile)?;

        let mut gpx = Gpx::default();
        gpx.version = GpxVersion::Gpx11;

        let filename: &str = ctx
            .primary_paths()
            .first()
            .unwrap()
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();
        let filestem: Vec<&str> = filename.split('.').collect();

        gpx_track.name = Some(filestem[0].to_string());
        // gpx_track.number = Some(1);
        gpx.tracks.push(gpx_track);

        gpx::write(&gpx, fd)?;
        info!("\"{}\" generated", gpxfile);
    }
    if cli.rtk_kml() {
        let kmlpath = workspace.join("rtk.kml");
        let kmlfile = kmlpath.to_string_lossy().to_string();
        let mut fd = File::create(&kmlfile)?;
        let kmldoc = KmlDocument {
            version: KmlVersion::V23,
            attrs: [(
                String::from("rtk-version"),
                env!("CARGO_PKG_VERSION").to_string(),
            )]
            .into_iter()
            .collect(),
            elements: kml_track,
        };
        let mut writer = KmlWriter::from_writer(&mut fd);
        writer.write(&Kml::KmlDocument(kmldoc))?;
        info!("\"{}\" generated", kmlfile);
    }
    Ok(())
}
