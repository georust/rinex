//! Meteo sensor
use crate::observable;
use crate::Observable;
use thiserror::Error;

/// Meteo Observation Sensor
#[derive(Default, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Sensor {
    /// Physics measured by this sensor
    pub observable: Observable,
    /// Model of this sensor
    pub model: Option<String>,
    /// Type of sensor
    pub sensor_type: Option<String>,
    /// Sensor accuracy [°C,..]
    pub accuracy: Option<f32>,
    /// Posible sensor location (ECEF) and possible
    /// height eccentricity
    pub position: Option<(f64, f64, f64, f64)>,
}

#[derive(Error, Debug)]
pub enum ParseSensorError {
    #[error("failed to identify observable")]
    ParseObservableError(#[from] observable::Error),
    #[error("failed to parse accuracy field")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}

impl std::str::FromStr for Sensor {
    type Err = ParseSensorError;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let (model, rem) = content.split_at(20);
        let (s_type, rem) = rem.split_at(20 + 6);
        let (accuracy, rem) = rem.split_at(7 + 4);
        let (observable, _) = rem.split_at(2);
        Ok(Self {
            model: {
                if model.trim().len() > 0 {
                    Some(model.trim().to_string())
                } else {
                    None
                }
            },
            sensor_type: {
                if s_type.trim().len() > 0 {
                    Some(s_type.trim().to_string())
                } else {
                    None
                }
            },
            accuracy: {
                if let Ok(f) = f32::from_str(accuracy.trim()) {
                    Some(f)
                } else {
                    None
                }
            },
            observable: Observable::from_str(observable.trim())?,
            position: None,
        })
    }
}

impl std::fmt::Display for Sensor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(model) = &self.model {
            write!(f, "{:<width$}", model, width = 20)?
        } else {
            write!(f, "{:20}", "")?;
        }

        if let Some(stype) = &self.sensor_type {
            write!(f, "{:<width$}", stype, width = 26)?;
        } else {
            write!(f, "{:26}", "")?;
        }

        if let Some(accuracy) = self.accuracy {
            write!(f, "{:^11.1}", accuracy)?
        } else {
            write!(f, "{:11}", "")?
        }
        write!(f, "{} SENSOR MOD/TYPE/ACC\n", self.observable)?;

        if let Some((x, y, z, h)) = self.position {
            write!(f, "{:14.4}", x)?;
            write!(f, "{:14.4}", y)?;
            write!(f, "{:14.4}", z)?;
            write!(f, "{:14.4}", h)?;
            write!(f, " {} SENSOR POS XYZ/H\n", self.observable)?
        }
        Ok(())
    }
}

impl Sensor {
    pub fn new(observable: Observable) -> Self {
        Self::default().with_observable(observable)
    }
    pub fn with_model(&self, model: &str) -> Self {
        let mut s = self.clone();
        s.model = Some(model.to_string());
        s
    }
    pub fn with_type(&self, stype: &str) -> Self {
        let mut s = self.clone();
        s.sensor_type = Some(stype.to_string());
        s
    }
    pub fn with_observable(&self, observable: Observable) -> Self {
        let mut s = self.clone();
        s.observable = observable;
        s
    }
    pub fn with_position(&self, pos: (f64, f64, f64, f64)) -> Self {
        let mut s = self.clone();
        s.position = Some(pos);
        s
    }
    pub fn with_accuracy(&self, accuracy: f32) -> Self {
        let mut s = self.clone();
        s.accuracy = Some(accuracy);
        s
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn to_str() {
        let s = Sensor::new(Observable::Temperature);
        assert_eq!(
            s.to_string(),
            "                                                         TD SENSOR MOD/TYPE/ACC\n"
        );
        let s = s.with_model("PAROSCIENTIFIC");
        assert_eq!(
            s.to_string(),
            "PAROSCIENTIFIC                                           TD SENSOR MOD/TYPE/ACC\n"
        );
        let s = s.with_observable(Observable::Pressure);
        let s = s.with_type("740-16B");
        assert_eq!(
            s.to_string(),
            "PAROSCIENTIFIC      740-16B                              PR SENSOR MOD/TYPE/ACC\n"
        );
        let s = s.with_accuracy(0.2);
        assert_eq!(
            s.to_string(),
            "PAROSCIENTIFIC      740-16B                       0.2    PR SENSOR MOD/TYPE/ACC\n"
        );

        let s = s.with_position((0.0, 0.0, 0.0, 1234.5678));
        assert_eq!(
            s.to_string(),
            "PAROSCIENTIFIC      740-16B                       0.2    PR SENSOR MOD/TYPE/ACC
        0.0000        0.0000        0.0000     1234.5678 PR SENSOR POS XYZ/H\n"
        );
    }
    #[test]
    fn from_str() {
        let s = Sensor::from_str("                                                  0.0    PR ");
        assert_eq!(s.is_ok(), true);
        let s = s.unwrap();
        assert_eq!(s.model, None);
        assert_eq!(s.sensor_type, None);
        assert_eq!(s.accuracy, Some(0.0));
        assert_eq!(s.observable, Observable::Pressure);

        let s = Sensor::from_str(
            "PAROSCIENTIFIC      740-16B                       0.2    PR SENSOR MOD/TYPE/ACC",
        );
        assert_eq!(s.is_ok(), true);
        let s = Sensor::from_str(
            "                                                  0.0    PR SENSOR MOD/TYPE/ACC",
        );
        assert_eq!(s.is_ok(), true);
    }
}
