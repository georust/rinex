use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::Write,
};

use crate::{
    cli::Context,
    fops::open_with_web_browser,
    graph::{build_3d_chart_epoch_label, build_chart_epoch_axis, PlotContext},
};
use clap::ArgMatches;

use thiserror::Error;

use hifitime::Epoch;
use rtk::prelude::PVTSolution;

extern crate gpx;
use gpx::{errors::GpxError, Gpx, GpxVersion, Waypoint};

use kml::{
    types::AltitudeMode, types::Coord as KmlCoord, types::Geometry as KmlGeometry,
    types::KmlDocument, types::Placemark, types::Point as KmlPoint, Kml, KmlVersion, KmlWriter,
};

extern crate geo_types;
use geo_types::Point as GeoPoint;

use plotly::{
    color::NamedColor,
    common::{Marker, MarkerSymbol, Mode, Visible},
    layout::MapboxStyle,
    ScatterMapbox,
};

use map_3d::{deg2rad, ecef2geodetic, rad2deg, Ellipsoid};

#[derive(Debug, Error)]
pub enum Error {
    #[error("std::io error")]
    IOError(#[from] std::io::Error),
    #[error("failed to generate gpx track")]
    GpxError(#[from] GpxError),
    #[error("failed to generate kml track")]
    KmlError(#[from] kml::Error),
}

// Customize HTML plot in case _apriori_ is known ahead of time
fn html_add_apriori_position(
    epochs: &Vec<Epoch>,
    solutions: &BTreeMap<Epoch, PVTSolution>,
    apriori_ecef: (f64, f64, f64),
    plot_ctx: &mut PlotContext,
) {
    let (x0, y0, z0) = apriori_ecef;
    let (lat0_rad, lon0_rad, _) = ecef2geodetic(x0, y0, z0, Ellipsoid::WGS84);
    let (lat0_ddeg, lon0_ddeg) = (rad2deg(lat0_rad), rad2deg(lon0_rad));

    // Mark _apriori_ position
    let apriori_scatter = ScatterMapbox::new(vec![lat0_ddeg], vec![lon0_ddeg])
        .marker(
            Marker::new()
                .size(6)
                .symbol(MarkerSymbol::Cross)
                .color(NamedColor::Red),
        )
        .name("Apriori");
    plot_ctx.add_trace(apriori_scatter);

    // |x-x*|, |y-y*|, |z-z*|  3D comparison
    let trace = build_3d_chart_epoch_label(
        "error",
        Mode::Markers,
        epochs.clone(),
        solutions
            .values()
            .map(|pvt| pvt.position.x - x0)
            .collect::<Vec<f64>>(),
        solutions
            .values()
            .map(|pvt| pvt.position.y - y0)
            .collect::<Vec<f64>>(),
        solutions
            .values()
            .map(|pvt| pvt.position.z - z0)
            .collect::<Vec<f64>>(),
    );

    plot_ctx.add_cartesian3d_plot(
        "Position errors",
        "x error [m]",
        "y error [m]",
        "z error [m]",
    );
    plot_ctx.add_trace(trace);

    // 2D |x-x*|, |y-y*| comparison
    plot_ctx.add_timedomain_2y_plot("X / Y Errors", "X error [m]", "Y error [m]");

    let trace = build_chart_epoch_axis(
        "x err",
        Mode::Markers,
        epochs.clone(),
        solutions
            .values()
            .map(|pvt| pvt.position.x - x0)
            .collect::<Vec<f64>>(),
    );
    plot_ctx.add_trace(trace);

    let trace = build_chart_epoch_axis(
        "y err",
        Mode::Markers,
        epochs.clone(),
        solutions
            .values()
            .map(|pvt| pvt.position.y - y0)
            .collect::<Vec<f64>>(),
    )
    .y_axis("y2");
    plot_ctx.add_trace(trace);

    // |z-z*| comparison
    plot_ctx.add_timedomain_plot("Altitude Errors", "Z error [m]");
    let trace = build_chart_epoch_axis(
        "z err",
        Mode::Markers,
        epochs.clone(),
        solutions
            .values()
            .map(|pvt| pvt.position.z - z0)
            .collect::<Vec<f64>>(),
    );
    plot_ctx.add_trace(trace);
}

pub fn post_process(
    ctx: &Context,
    solutions: BTreeMap<Epoch, PVTSolution>,
    matches: &ArgMatches,
) -> Result<(), Error> {
    // create a dedicated plot context
    let mut plot_ctx = PlotContext::new();

    // Convert solutions to geodetic DDEG
    let (mut lat, mut lon) = (Vec::<f64>::new(), Vec::<f64>::new());
    for solution in solutions.values() {
        let (lat_rad, lon_rad, _) = ecef2geodetic(
            solution.position.x,
            solution.position.y,
            solution.position.z,
            Ellipsoid::WGS84,
        );
        lat.push(rad2deg(lat_rad));
        lon.push(rad2deg(lon_rad));
    }

    let nb_solutions = solutions.len();
    let final_solution_ddeg = (lat[nb_solutions - 1], lon[nb_solutions - 1]);
    let epochs = solutions.keys().copied().collect::<Vec<Epoch>>();

    let lat0_rad = if let Some(apriori_ecef) = ctx.rx_ecef {
        ecef2geodetic(
            apriori_ecef.0,
            apriori_ecef.1,
            apriori_ecef.2,
            Ellipsoid::WGS84,
        )
        .0
    } else {
        deg2rad(final_solution_ddeg.0)
    };

    let lon0_rad = if let Some(apriori_ecef) = ctx.rx_ecef {
        ecef2geodetic(
            apriori_ecef.0,
            apriori_ecef.1,
            apriori_ecef.2,
            Ellipsoid::WGS84,
        )
        .1
    } else {
        deg2rad(final_solution_ddeg.1)
    };

    // TODO: would be nice to adapt zoom level
    plot_ctx.add_world_map(
        "PVT solutions",
        true, // show legend
        MapboxStyle::OpenStreetMap,
        final_solution_ddeg, //center
        18,                  // zoom
    );

    // Process solutions
    // Scatter Mapbox (map projection)
    let mut prev_pct = 0;
    for (index, (_, sol_i)) in solutions.iter().enumerate() {
        let (lat_rad, lon_rad, _) = ecef2geodetic(
            sol_i.position.x,
            sol_i.position.y,
            sol_i.position.z,
            Ellipsoid::WGS84,
        );
        let (lat_ddeg, lon_ddeg) = (rad2deg(lat_rad), rad2deg(lon_rad));

        let pct = index * 100 / nb_solutions;
        if pct % 10 == 0 && index > 0 && pct != prev_pct || index == nb_solutions - 1 {
            let (title, visible) = if index == nb_solutions - 1 {
                ("FINAL".to_string(), Visible::True)
            } else {
                (format!("Solver: {:02}%", pct), Visible::LegendOnly)
            };
            let pvt_scatter = ScatterMapbox::new(vec![lat_ddeg], vec![lon_ddeg])
                .marker(
                    Marker::new()
                        .size(5)
                        .color(NamedColor::Black)
                        .symbol(MarkerSymbol::Circle),
                )
                .visible(visible)
                .name(&title);
            plot_ctx.add_trace(pvt_scatter);
        }
        prev_pct = pct;
    }

    // Absolute velocity
    plot_ctx.add_timedomain_2y_plot("Velocity (X & Y)", "Speed [m/s]", "Speed [m/s]");
    let trace = build_chart_epoch_axis(
        "velocity (x)",
        Mode::Markers,
        epochs.clone(),
        solutions
            .values()
            .map(|pvt| pvt.velocity.x)
            .collect::<Vec<f64>>(),
    );
    plot_ctx.add_trace(trace);

    let trace = build_chart_epoch_axis(
        "velocity (y)",
        Mode::Markers,
        epochs.clone(),
        solutions
            .values()
            .map(|pvt| pvt.velocity.y)
            .collect::<Vec<f64>>(),
    )
    .y_axis("y2");
    plot_ctx.add_trace(trace);

    plot_ctx.add_timedomain_plot("Velocity (Z)", "Speed [m/s]");
    let trace = build_chart_epoch_axis(
        "velocity (z)",
        Mode::Markers,
        epochs.clone(),
        solutions
            .values()
            .map(|pvt| pvt.velocity.z)
            .collect::<Vec<f64>>(),
    );
    plot_ctx.add_trace(trace);

    plot_ctx.add_timedomain_2y_plot("Clock offset", "dt [s]", "TDOP [s]");
    let trace = build_chart_epoch_axis(
        "dt",
        Mode::Markers,
        epochs.clone(),
        solutions
            .values()
            .map(|pvt| pvt.dt.to_seconds())
            .collect::<Vec<f64>>(),
    );
    plot_ctx.add_trace(trace);

    let trace = build_chart_epoch_axis(
        "tdop",
        Mode::Markers,
        epochs.clone(),
        solutions.values().map(|pvt| pvt.tdop).collect::<Vec<f64>>(),
    )
    .y_axis("y2");
    plot_ctx.add_trace(trace);

    // Dilution of Precision
    plot_ctx.add_timedomain_plot("GDOP", "GDOP [m]");
    let trace = build_chart_epoch_axis(
        "gdop",
        Mode::Markers,
        epochs.clone(),
        solutions.values().map(|pvt| pvt.gdop).collect::<Vec<f64>>(),
    );
    plot_ctx.add_trace(trace);

    plot_ctx.add_timedomain_2y_plot("HDOP, VDOP", "HDOP [m]", "VDOP [m]");
    let trace = build_chart_epoch_axis(
        "hdop",
        Mode::Markers,
        epochs.clone(),
        solutions
            .values()
            .map(|pvt| pvt.hdop(lat0_rad, lon0_rad))
            .collect::<Vec<f64>>(),
    );
    plot_ctx.add_trace(trace);

    let trace = build_chart_epoch_axis(
        "vdop",
        Mode::Markers,
        epochs.clone(),
        solutions
            .values()
            .map(|pvt| pvt.vdop(lat0_rad, lon0_rad))
            .collect::<Vec<f64>>(),
    )
    .y_axis("y2");
    plot_ctx.add_trace(trace);

    if let Some(apriori_ecef) = ctx.rx_ecef {
        html_add_apriori_position(&epochs, &solutions, apriori_ecef, &mut plot_ctx);
    }

    // render plots
    let graphs = ctx.workspace.join("Solutions.html");
    let graphs = graphs.to_string_lossy().to_string();
    let mut fd = File::create(&graphs).unwrap_or_else(|_| panic!("failed to crate \"{}\"", graphs));
    write!(fd, "{}", plot_ctx.to_html()).expect("failed to render PVT solutions");
    info!("\"{}\" solutions generated", graphs);

    /*
     * Generate txt, GPX, KML..
     */
    let txtpath = ctx.workspace.join("solutions.csv");
    let txtfile = txtpath.to_string_lossy().to_string();
    let mut fd = File::create(&txtfile)?;

    let mut gpx_track = gpx::Track::default();
    let mut kml_track = Vec::<Kml>::new();

    writeln!(
        fd,
        "Epoch, x_ecef, y_ecef, z_ecef, speed_x, speed_y, speed_z, hdop, vdop, rcvr_clock_bias, tdop"
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
    info!("\"{}\" generated", txtfile);
    if matches.get_flag("gpx") {
        let prefix = ctx.name.clone();
        let gpxpath = ctx.workspace.join(format!("{}.gpx", prefix));
        let gpxfile = gpxpath.to_string_lossy().to_string();

        let fd = File::create(&gpxfile)?;

        let mut gpx = Gpx::default();
        gpx.version = GpxVersion::Gpx11;
        gpx_track.name = Some(prefix.clone());
        // gpx_track.number = Some(1);
        gpx.tracks.push(gpx_track);

        gpx::write(&gpx, fd)?;
        info!("{} gpx track generated", gpxfile);
    }
    if matches.get_flag("kml") {
        let prefix = ctx.name.clone();
        let kmlpath = ctx.workspace.join(format!("{}.kml", prefix));
        let kmlfile = kmlpath.to_string_lossy().to_string();

        let mut fd = File::create(&kmlfile)?;

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
        info!("{} kml track generated", kmlfile);
    }

    if !ctx.quiet {
        let graphs = ctx.workspace.join("Solutions.html");
        let graphs = graphs.to_string_lossy().to_string();
        open_with_web_browser(&graphs);
    }

    Ok(())
}
