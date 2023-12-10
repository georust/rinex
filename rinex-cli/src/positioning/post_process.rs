use crate::cli::Cli;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use thiserror::Error;

use hifitime::Epoch;
use rinex::prelude::RnxContext;
use rtk::prelude::PVTSolution;

extern crate gpx;
use gpx::{errors::GpxError, Gpx, GpxVersion, Waypoint};

use kml::{
    types::AltitudeMode, types::Coord as KmlCoord, types::Geometry as KmlGeometry,
    types::KmlDocument, types::Placemark, types::Point as KmlPoint, Kml, KmlVersion, KmlWriter,
};

extern crate geo_types;
use geo_types::Point as GeoPoint;

use crate::context_stem;

use plotly::color::NamedColor;
use plotly::common::Mode;
use plotly::common::{Marker, MarkerSymbol};
use plotly::layout::MapboxStyle;
use plotly::ScatterMapbox;

use crate::fops::open_with_web_browser;
use crate::plot::{build_3d_chart_epoch_label, build_chart_epoch_axis, PlotContext};
use map_3d::{ecef2geodetic, rad2deg, Ellipsoid};

#[derive(Debug, Error)]
pub enum Error {
    #[error("std::io error")]
    IOError(#[from] std::io::Error),
    #[error("failed to generate gpx track")]
    GpxError(#[from] GpxError),
    #[error("failed to generate kml track")]
    KmlError(#[from] kml::Error),
}

pub fn post_process(
    workspace: PathBuf,
    cli: &Cli,
    ctx: &RnxContext,
    results: BTreeMap<Epoch, PVTSolution>,
) -> Result<(), Error> {
    // create a dedicated plot context
    let no_graph = cli.no_graph();
    let mut plot_ctx = PlotContext::new();

    let (x, y, z) = ctx
        .ground_position()
        .unwrap() // cannot fail at this point
        .to_ecef_wgs84();

    let (lat_ddeg, lon_ddeg, _) = ctx
        .ground_position()
        .unwrap() // cannot fail at this point
        .to_geodetic();

    if !no_graph {
        let epochs = results.keys().copied().collect::<Vec<Epoch>>();

        let (mut lat, mut lon) = (Vec::<f64>::new(), Vec::<f64>::new());
        for result in results.values() {
            let px = x + result.pos.x;
            let py = y + result.pos.y;
            let pz = z + result.pos.z;
            let (lat_ddeg, lon_ddeg, _) = ecef2geodetic(px, py, pz, Ellipsoid::WGS84);
            lat.push(rad2deg(lat_ddeg));
            lon.push(rad2deg(lon_ddeg));
        }

        plot_ctx.add_world_map(
            "PVT solutions",
            true, // show legend
            MapboxStyle::OpenStreetMap,
            (lat_ddeg, lon_ddeg), //center
            18,                   // zoom in!!
        );

        let ref_scatter = ScatterMapbox::new(vec![lat_ddeg], vec![lon_ddeg])
            .marker(
                Marker::new()
                    .size(5)
                    .symbol(MarkerSymbol::Circle)
                    .color(NamedColor::Red),
            )
            .name("Apriori");
        plot_ctx.add_trace(ref_scatter);

        let pvt_scatter = ScatterMapbox::new(lat, lon)
            .marker(
                Marker::new()
                    .size(5)
                    .symbol(MarkerSymbol::Circle)
                    .color(NamedColor::Black),
            )
            .name("PVT");
        plot_ctx.add_trace(pvt_scatter);

        let trace = build_3d_chart_epoch_label(
            "error",
            Mode::Markers,
            epochs.clone(),
            results.values().map(|e| e.pos.x).collect::<Vec<f64>>(),
            results.values().map(|e| e.pos.y).collect::<Vec<f64>>(),
            results.values().map(|e| e.pos.z).collect::<Vec<f64>>(),
        );

        /*
         * Create graphical visualization
         * dx, dy : one dual plot
         * hdop, vdop : one dual plot
         * dz : dedicated plot
         * dt, tdop : one dual plot
         */
        plot_ctx.add_cartesian3d_plot(
            "Position errors",
            "x error [m]",
            "y error [m]",
            "z error [m]",
        );
        plot_ctx.add_trace(trace);

        plot_ctx.add_timedomain_2y_plot("Velocity (X & Y)", "Speed [m/s]", "Speed [m/s]");
        let trace = build_chart_epoch_axis(
            "velocity (x)",
            Mode::Markers,
            epochs.clone(),
            results.values().map(|p| p.vel.x).collect::<Vec<f64>>(),
        );
        plot_ctx.add_trace(trace);

        let trace = build_chart_epoch_axis(
            "velocity (y)",
            Mode::Markers,
            epochs.clone(),
            results.values().map(|p| p.vel.y).collect::<Vec<f64>>(),
        )
        .y_axis("y2");
        plot_ctx.add_trace(trace);

        plot_ctx.add_timedomain_plot("Velocity (Z)", "Speed [m/s]");
        let trace = build_chart_epoch_axis(
            "velocity (z)",
            Mode::Markers,
            epochs.clone(),
            results.values().map(|p| p.vel.z).collect::<Vec<f64>>(),
        );
        plot_ctx.add_trace(trace);

        plot_ctx.add_timedomain_plot("GDOP", "GDOP [m]");
        let trace = build_chart_epoch_axis(
            "gdop",
            Mode::Markers,
            epochs.clone(),
            results.values().map(|e| e.gdop()).collect::<Vec<f64>>(),
        );
        plot_ctx.add_trace(trace);

        plot_ctx.add_timedomain_2y_plot("HDOP, VDOP", "HDOP [m]", "VDOP [m]");
        let trace = build_chart_epoch_axis(
            "hdop",
            Mode::Markers,
            epochs.clone(),
            results
                .values()
                .map(|e| e.hdop(lat_ddeg, lon_ddeg))
                .collect::<Vec<f64>>(),
        );
        plot_ctx.add_trace(trace);

        let trace = build_chart_epoch_axis(
            "vdop",
            Mode::Markers,
            epochs.clone(),
            results
                .values()
                .map(|e| e.vdop(lat_ddeg, lon_ddeg))
                .collect::<Vec<f64>>(),
        )
        .y_axis("y2");
        plot_ctx.add_trace(trace);

        plot_ctx.add_timedomain_2y_plot("Clock offset", "dt [s]", "TDOP [s]");
        let trace = build_chart_epoch_axis(
            "dt",
            Mode::Markers,
            epochs.clone(),
            results.values().map(|e| e.dt).collect::<Vec<f64>>(),
        );
        plot_ctx.add_trace(trace);

        let trace = build_chart_epoch_axis(
            "tdop",
            Mode::Markers,
            epochs.clone(),
            results.values().map(|e| e.tdop()).collect::<Vec<f64>>(),
        )
        .y_axis("y2");
        plot_ctx.add_trace(trace);

        // render plots
        let graphs = workspace.join("rtk.html");
        let graphs = graphs.to_string_lossy().to_string();
        let mut fd =
            File::create(&graphs).unwrap_or_else(|_| panic!("failed to crate \"{}\"", graphs));
        write!(fd, "{}", plot_ctx.to_html()).expect("failed to render rtk visualization");
        info!("\"{}\" rtk view generated", graphs);
    }

    /*
     * Generate txt, GPX, KML..
     */
    let txtpath = workspace.join("pvt-solutions.csv");
    let txtfile = txtpath.to_string_lossy().to_string();
    let mut fd = File::create(&txtfile)?;

    let mut gpx_track = gpx::Track::default();
    let mut kml_track = Vec::<Kml>::new();

    writeln!(
        fd,
        "Epoch, dx, dy, dz, x_ecef, y_ecef, z_ecef, speed_x, speed_y, speed_z, hdop, vdop, rcvr_clock_bias, tdop"
    )?;

    for (epoch, solution) in results {
        let (px, py, pz) = (x + solution.pos.x, y + solution.pos.y, z + solution.pos.z);
        let (lat, lon, alt) = map_3d::ecef2geodetic(px, py, pz, map_3d::Ellipsoid::WGS84);
        let (hdop, vdop, tdop) = (
            solution.hdop(lat_ddeg, lon_ddeg),
            solution.vdop(lat_ddeg, lon_ddeg),
            solution.tdop(),
        );
        writeln!(
            fd,
            "{:?}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}",
            epoch,
            solution.pos.x,
            solution.pos.y,
            solution.pos.z,
            px,
            py,
            pz,
            solution.vel.x,
            solution.vel.y,
            solution.vel.z,
            hdop,
            vdop,
            solution.dt,
            tdop
        )?;
        if cli.gpx() {
            let mut segment = gpx::TrackSegment::new();
            let mut wp = Waypoint::new(GeoPoint::new(rad2deg(lat), rad2deg(lon)));
            wp.elevation = Some(alt);
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
        if cli.kml() {
            kml_track.push(Kml::Placemark(Placemark {
                name: Some(format!("{:?}", epoch)),
                description: Some(String::from("\"Receiver Location\"")),
                geometry: {
                    Some(KmlGeometry::Point(KmlPoint {
                        coord: {
                            KmlCoord {
                                x: rad2deg(lat),
                                y: rad2deg(lon),
                                z: Some(alt),
                            }
                        },
                        extrude: false,
                        altitude_mode: AltitudeMode::Absolute,
                        attrs: HashMap::new(),
                    }))
                },
                attrs: [(String::from("TDOP"), format!("{:.6E}", solution.tdop()))]
                    .into_iter()
                    .collect(),
                children: vec![],
            }));
        }
    }
    info!("\"{}\" generated", txtfile);
    if cli.gpx() {
        let gpxpath = workspace.join(&format!("{}.gpx", context_stem(&ctx)));
        let gpxfile = gpxpath.to_string_lossy().to_string();
        let fd = File::create(&gpxfile)?;

        let mut gpx = Gpx::default();
        gpx.version = GpxVersion::Gpx11;
        gpx_track.name = Some(context_stem(ctx).to_string());
        // gpx_track.number = Some(1);
        gpx.tracks.push(gpx_track);

        gpx::write(&gpx, fd)?;
        info!("{} gpx track generated", gpxfile);
    }
    if cli.kml() {
        let kmlpath = workspace.join(&format!("{}.kml", context_stem(&ctx)));
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
            elements: {
                vec![Kml::Folder {
                    attrs: HashMap::new(),
                    elements: kml_track,
                }]
            },
        };
        let mut writer = KmlWriter::from_writer(&mut fd);
        writer.write(&Kml::KmlDocument(kmldoc))?;
        info!("{} kml track generated", kmlfile);
    }

    if !cli.quiet() && !no_graph {
        let graphs = workspace.join("rtk.html");
        let graphs = graphs.to_string_lossy().to_string();
        open_with_web_browser(&graphs);
    }

    Ok(())
}
