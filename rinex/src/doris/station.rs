//! DORIS Station

use crate::{domes::Domes, doris::Error};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Station {
    /// Station mnemonic label (Antenna point)
    pub label: String,
    /// DORIS site name
    pub site: String,
    /// DOMES site identifier
    pub domes: Domes,
    /// Beacon generation
    pub gen: u8,
    /// K frequency shift factor
    pub k_factor: i8,
    /// ID used in this file indexing
    pub(crate) key: u16,
}

impl Station {
    const USO_FREQ: f64 = 5.0E6_f64;
    /// Station S1 Frequency shift factor
    pub fn s1_frequency_shift(&self) -> f64 {
        543.0 * Self::USO_FREQ * (3.0 / 4.0 + 87.0 * self.k_factor as f64 / 5.0 * 2.0_f64.powi(26))
    }
    /// Station U2 Frequency shift factor
    pub fn u2_frequency_shift(&self) -> f64 {
        107.0 * Self::USO_FREQ * (3.0 / 4.0 + 87.0 * self.k_factor as f64 / 5.0 * 2.0_f64.powi(26))
    }
}

/*
 * Parses DORIS station, returns ID# code and Station
 */
impl std::str::FromStr for Station {
    type Err = Error;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        if content.len() < 40 {
            return Err(Error::InvalidStation);
        }

        let content = content.split_at(1).1;
        let (key, rem) = content.split_at(4);
        let (label, rem) = rem.split_at(5);
        let (name, rem) = rem.split_at(30);
        let (domes, rem) = rem.split_at(10);
        let (gen, rem) = rem.split_at(3);
        let (k_factor, _) = rem.split_at(3);

        Ok(Station {
            site: name.trim().to_string(),
            label: label.trim().to_string(),
            domes: Domes::from_str(domes.trim())?,
            gen: gen
                .trim()
                .parse::<u8>()
                .or(Err(Error::BeaconGenerationParsing))?,
            k_factor: k_factor.trim().parse::<i8>().or(Err(Error::KfParsing))?,
            key: key.trim().parse::<u16>().or(Err(Error::IdParsing))?,
        })
    }
}

impl std::fmt::Display for Station {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "D{:02}  {} {:<29} {}  {}   {}",
            self.key, self.label, self.site, self.domes, self.gen, self.k_factor
        )
    }
}

#[cfg(test)]
mod test {
    use super::Station;
    use crate::domes::{Domes, TrackingPoint as DomesTrackingPoint};
    use std::str::FromStr;
    #[test]
    fn station_parsing() {
        for (desc, expected) in [
            (
                "D01  OWFC OWENGA                        50253S002  3   0",
                Station {
                    label: "OWFC".to_string(),
                    site: "OWENGA".to_string(),
                    domes: Domes {
                        area: 502,
                        site: 53,
                        sequential: 2,
                        point: DomesTrackingPoint::Instrument,
                    },
                    gen: 3,
                    k_factor: 0,
                    key: 1,
                },
            ),
            (
                "D17  GRFB GREENBELT                     40451S178  3   0",
                Station {
                    label: "GRFB".to_string(),
                    site: "GREENBELT".to_string(),
                    domes: Domes {
                        area: 404,
                        site: 51,
                        sequential: 178,
                        point: DomesTrackingPoint::Instrument,
                    },
                    gen: 3,
                    k_factor: 0,
                    key: 17,
                },
            ),
        ] {
            let station = Station::from_str(desc).unwrap();
            assert_eq!(station, expected, "station parsing error");
            assert_eq!(station.to_string(), desc, "station reciprocal error");
        }
    }
}
