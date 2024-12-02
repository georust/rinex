//! IONEX maps parsing

use crate::{
    ionex::{IonexKey, IonexMapCoordinates, Record, TEC},
    prelude::{Epoch, ParsingError},
};

pub fn is_new_tec_map(line: &str) -> bool {
    line.contains("START OF TEC MAP")
}

pub fn is_new_rms_map(line: &str) -> bool {
    line.contains("START OF RMS MAP")
}

pub fn is_new_height_map(line: &str) -> bool {
    line.contains("START OF HEIGHT MAP")
}

/// Parses Map grid to follow, returns
/// - fixed_latitude [ddeg]
/// - long1 [ddeg]
/// - long_spacing [ddeg]
/// - fixed_altitude [km]
fn parse_grid_specs(line: &str) -> Result<(f64, f64, f64, f64), ParsingError> {
    let (fixed_lat, rem) = line[2..].split_at(6);

    let fixed_lat = fixed_lat
        .trim()
        .parse::<f64>()
        .map_err(|_| ParsingError::IonexGridCoordinates)?;

    let (long1, rem) = rem.split_at(6);

    let long1 = long1
        .trim()
        .parse::<f64>()
        .map_err(|_| ParsingError::IonexGridCoordinates)?;

    // lon2 field is not used, we iterate using spacing starting @long1
    let (_, rem) = rem.split_at(6);

    let (long_spacing, rem) = rem.split_at(6);

    let long_spacing = long_spacing
        .trim()
        .parse::<f64>()
        .map_err(|_| ParsingError::IonexGridCoordinates)?;

    let (fixed_alt, _) = rem.split_at(6);

    let fixed_alt = fixed_alt
        .trim()
        .parse::<f64>()
        .map_err(|_| ParsingError::IonexGridCoordinates)?;

    return Ok((fixed_lat, long1, long_spacing, fixed_alt));
}

pub fn parse_rms_map() {}
pub fn parse_height_map() {}

/// Parses all maps contained in following TEC description.
/// This describes a serie of TEC point in volume.
/// ## Inputs
///   - content: readable content (ASCII UTF-8)
///   - lat_exponent: deduced from IONEX header for coordinates quantization
///   - long_exponent: deduced from IONEX header for coordinates quantization
///   - tec_exponent: kept up to date, for correct data interpretation
///   - epoch: kept up to date, for correct classification
pub fn parse_tec_map(
    content: &str,
    lat_exponent: i8,
    long_exponent: i8,
    alt_exponent: i8,
    tec_exponent: i8,
    mut epoch: Epoch,
    record: &mut Record,
) -> Result<(), ParsingError> {
    let lines = content.lines();
    let mut epoch = Epoch::default();

    let mut fixed_lat = 0.0_f64;
    let (mut long1, mut long_spacing) = (0.0_f64, 0.0_f64);
    let mut fixed_alt = 0.0_f64;

    let mut long = 0.0_f64; // current longitude (pointer)

    for line in lines {
        if line.len() > 60 {
            let (content, marker) = line.split_at(60);

            // Handle special cases
            // * data scaler update
            // * Timestamp specs (Epoch)
            if marker.contains("EXPONENT") {
                // should not have been presented (handled @ higher level)
                continue; // avoid parsing
            } else if marker.contains("EPOCH OF CURRENT MAP") {
                // should not have been presented (handled @ higher level)
                continue; // avoid parsing
            } else if marker.contains("START OF") {
                continue; // avoid parsing
            } else if marker.contains("LAT/LON1/LON2/DLON/H") {
                // grid specs (to follow)
                (fixed_lat, long1, long_spacing, fixed_alt) = parse_grid_specs(content)?;

                // determine quantization exponents

                long = long1; // reset pointer
                continue; // avoid parsing
            } else if marker.contains("END OF TEC MAP") {
                // block conclusion
                // don't care about block #id actually
                return Ok(());
            }
        }

        // proceed to parsing
        for item in line.split_ascii_whitespace() {
            // data interpretation
            if let Ok(tec) = item.trim().parse::<i32>() {
                let tec = TEC::from_quantized(tec, tec_exponent);

                let coordinates = IonexMapCoordinates::new(
                    fixed_lat,
                    lat_exponent,
                    long,
                    long_exponent,
                    fixed_alt,
                    alt_exponent,
                );

                let key = IonexKey { epoch, coordinates };

                record.insert(key, tec);
            }

            long += long_spacing;
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::{
        is_new_height_map, is_new_rms_map, is_new_tec_map, parse_grid_specs, parse_tec_map,
    };

    use crate::{
        ionex::{IonexKey, IonexMapCoordinates, Record},
        prelude::Epoch,
    };

    #[test]
    fn new_ionex_map() {
        assert!(is_new_tec_map(
            "1                                                      START OF TEC MAP"
        ));

        assert!(is_new_rms_map(
            "1                                                      START OF RMS MAP"
        ));

        assert!(is_new_height_map(
            "1                                                   START OF HEIGHT MAP"
        ));
    }

    #[test]
    fn grid_specs_parsing() {
        let content =
            "    87.5-180.0 180.0   5.0 450.0                            LAT/LON1/LON2/DLON/H";
        let (fixed_lat, long1, long_spacing, fixed_alt) = parse_grid_specs(content).unwrap();

        assert_eq!(fixed_lat, 87.5);
        assert_eq!(long1, -180.0);
        assert_eq!(long_spacing, 5.0);
        assert_eq!(fixed_alt, 450.0);
    }

    #[test]
    fn tec_map_parsing() {
        let mut record = Record::default();

        let lat_exponent = 1;
        let long_exponent = 0;
        let alt_exponent = 0;
        let tec_exponent = -1;
        let epoch = Epoch::from_gregorian_utc_at_midnight(2017, 1, 1);

        let content =
            "     1                                                      START OF TEC MAP    
  2017     1     1     0     0     0                        EPOCH OF CURRENT MAP
    87.5-180.0 180.0   5.0 450.0                            LAT/LON1/LON2/DLON/H
   33   33   32   32   32   31   31   30   30   30   29   29   28   28   28   27
   27   27   26   26   26   26   26   26   26   26   26   26   26   26   26   26
   27   27   27   28   28   29   29   30   30   31   31   32   32   33   33   33
   34   34   35   35   35   35   36   36   36   36   36   36   36   36   36   35
   35   35   35   35   34   34   34   33   33
    85.0-180.0 180.0   5.0 450.0                            LAT/LON1/LON2/DLON/H
   36   36   35   35   34   34   33   33   32   31   31   30   29   28   28   27
   26   25   25   24   24   23   23   22   22   22   22   22   22   23   23   24
   24   25   25   26   27   28   29   29   30   31   32   33   34   35   36   37
   38   39   39   40   41   41   41   41   42   42   42   41   41   41   41   40
   40   40   39   39   38   38   37   37   36
     1                                                      END OF TEC MAP      ";

        parse_tec_map(
            content,
            lat_exponent,
            long_exponent,
            alt_exponent,
            tec_exponent,
            epoch,
            &mut record,
        )
        .unwrap();

        let coordinates = IonexMapCoordinates::new(
            87.5,
            lat_exponent,
            -180.0,
            long_exponent,
            450.0,
            alt_exponent,
        );

        let key = IonexKey { epoch, coordinates };

        panic!("{:#?}", record);

        let tec = record
            .get(&key)
            .expect(&format!("missing value for {:?} data", key));
    }
}
