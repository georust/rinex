use kml::{Kml, KmlWriter};
use rinex::prelude::*;

mod cli;
use cli::{Cli, CliError};
use log::{info, warn};

use kml::{
    types::{AltitudeMode, Coord, LineString, LinearRing, Polygon, Placemark, Geometry},
    KmlDocument,
};
use std::collections::HashMap;



//use std::io::Write;

fn main() -> Result<(), CliError> {
    pretty_env_logger::init_timed();

    let cli = Cli::new();

    let fp = cli.ionex_filepath();
    info!("reading {}", fp);
    let rinex = Rinex::from_file(fp)?;

    if !rinex.is_ionex() {
        warn!("input file is not a ionex file");
        return Err(CliError::FileTypeError(format!(
            "{:?}",
            rinex.header.rinex_type
        )));
    }

    let mut kml_doc: KmlDocument<f64> = KmlDocument::default();
    kml_doc.version = cli.kml_version();
    if let Some(attrs) = cli.kml_attributes() {
        kml_doc.attrs = attrs;
    }

    let record = rinex.record.as_ionex().unwrap();

    let mut buf = std::io::stdout().lock();
    let mut writer = KmlWriter::<_, f64>::from_writer(&mut buf);

    // We wrap each Epoch in separate "Folders"
    for (epoch, (_map, _, _)) in record {
        let mut epoch_folder: Vec<Kml<f64>> = Vec::new();
        let epoch_folder_attrs = vec![(String::from("Epoch"), epoch.to_string())]
            .into_iter()
            .collect::<HashMap<String, String>>();

        //test a linestring to describe equipoential TECu area
        let polygon = Polygon::<f64> {
            inner: vec![],
            outer: {
                LinearRing::<f64> {
                    coords: vec![
                        Coord {
                            x: 4.119067147539055, 
                            y: 43.73425044812969,
                            z: None,
                        },
                        Coord {
                            x: 4.11327766588697,
                            y: 43.73124529989733,
                            z: None,
                        },
                        Coord {
                            x: 4.119067147539055,
                            y: 43.73425044812969,
                            z: None,
                        },
                        Coord {
                            x: 4.129067147539055,
                            y: 44.73425044812969,
                            z: None,
                        },
                        Coord {
                            x: 4.109067147539055,
                            y: 44.73425044812969,
                            z: None,
                        },
                    ],
                    extrude: false,
                    tessellate: true,
                    altitude_mode: AltitudeMode::RelativeToGround,
                    attrs: vec![(String::from("name"), String::from("test"))]
                    .into_iter()
                    .collect(),
                }
            },
            extrude: false,
            tessellate: true,
            altitude_mode: AltitudeMode::RelativeToGround,
            attrs: vec![(String::from("name"), String::from("test"))]
            .into_iter()
            .collect(),
        };

        let placemark = Placemark::<f64> {
            name: Some(String::from("This is a test")),
            description: Some(String::from("Great description")),
            geometry: Some(Geometry::Polygon(polygon)),
            children: vec![],
            attrs: HashMap::new(),
        };

        epoch_folder.push(Kml::Placemark(placemark));

        //    // We wrap equipotential TECu areas in
        //    // we wrap altitude levels in separate "Folders"
        //    // in 2D IONEX (single altitude value): you only get one folder per Epoch
        //    for (h, maps) in rinex.ionex() {
        //        let folder = Folder::default();
        //        folder.attrs =
        //            attrs: vec![("elevation", format!("{:.3e}", h)].into_iter().collect()
        //        };
        //        for (potential, boundaries) in maps.tec_equipotential(10) {
        //            // define one color for following areas
        //            let color = colorous::linear(percentile);
        //            let style = LineStyle {
        //                id: Some(percentile.to_string()),
        //                width: 1.0_f64,
        //                color_mode: ColorMode::Default,
        //                attrs: vec![("percentile", format!("{}", percentile)].into_iter().collect(),
        //            };
        //            folder.elements.push(style);
        //            folder.elements.push(boundaries);
        //        }
        //        kml.elements.push(folder);
        //    }
        //folder_content..push(epoch_folder);

        let epoch_folder: Kml<f64> = Kml::Folder {
            attrs: epoch_folder_attrs,
            elements: epoch_folder,
        };
        // add folder to document
        kml_doc.elements.push(epoch_folder);

        break;//TODO
    }

    // generate document
    let kml = Kml::KmlDocument(kml_doc);
    writer.write(&kml)?;
    info!("kml generated");
    
    Ok(())
}
