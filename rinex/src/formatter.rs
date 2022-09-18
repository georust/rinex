#[cfg(feature = "serde")]
pub mod point3d {
    //use super::ParseError;
    use std::str::FromStr;
    use serde::{Deserializer, Deserialize, de::Error};
    
    pub enum ParseError {
        /// Failed to parse (x, y, z) triplet
        #[cfg(feature = "serde")]
        Point3dXyz, 
    }

    impl std::fmt::Display for ParseError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                Self::Point3dXyz => write!(f, "failed to parse (x,y,z) triplet"),
            }
        }
    }

    /// Dumps a rust_3d::Point3D structure
    pub fn serialize<S>(p: rust_3d::Point3D, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = format!("{},{},{}",p.x,p.y,p.z); 
        serializer.serialize_str(&s)
    }
    /// Parses a rust_3d::Point3D structure
    pub fn deserialize<'de, D>(deserializer: D) -> Result<rust_3d::Point3D, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let items: Vec<&str> = s.split(",").collect();
        if let Ok(x) = f64::from_str(items[0]) {
            if let Ok(y) = f64::from_str(items[1]) {
                if let Ok(z) = f64::from_str(items[2]) {
                    return Ok(rust_3d::Point3D {x, y, z })
                }
            }
        }
        Err(ParseError::Point3dXyz)
            .map_err(Error::custom)
    }
}

#[cfg(feature = "serde")]
pub mod opt_point3d {
    use std::str::FromStr;
    use serde::{Serializer, Deserializer, Deserialize};
    /// Dumps an optionnal rust_3d::Point3D
    pub fn serialize<S>(p: &Option<rust_3d::Point3D>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(p) = p {
            let s = format!("{},{},{}", p.x, p.y, p.z); 
            serializer.serialize_str(&s)
        } else {
            serializer.serialize_str("")
        }
    }
    /// Parses an optionnal chrono::NaiveDateTime structure
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<rust_3d::Point3D>, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let Ok(s) = String::deserialize(deserializer) {
            let items: Vec<&str> = s.split(",").collect();
            if let Ok(x) = f64::from_str(items[0]) {
                if let Ok(y) = f64::from_str(items[1]) {
                    if let Ok(z) = f64::from_str(items[2]) {
                        return Ok(Some(rust_3d::Point3D::new(x,y,z)))
                    }
                }
            }
        }
        Ok(None)
    }
}

#[cfg(feature = "serde")]
pub mod datetime {
    use serde::{Serializer, Deserializer, Deserialize, de::Error};
    /// Dumps a chrono::NaiveDateTime structure
    pub fn serialize<S>(datetime: &chrono::NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", datetime.format("%Y-%m-%d %H:%M:%S"));
        serializer.serialize_str(&s)
    }
    /// Parses a chrono::NaiveDateTime structure
    pub fn deserialize<'de, D>(deserializer: D) -> Result<chrono::NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>, 
    {
        let s = String::deserialize(deserializer)?;
        chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
            .map_err(Error::custom)
    }
}

#[cfg(feature = "serde")]
pub mod opt_datetime {
    use serde::{Serializer, Deserializer, Deserialize};
    /// Dumps an optionnal chrono::NaiveDateTime structure
    pub fn serialize<S>(datetime: &Option<chrono::NaiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(datetime) = datetime {
            let s = format!("{}", datetime.format("%Y-%m-%d %H:%M:%S"));
            serializer.serialize_str(&s)
        } else {
            serializer.serialize_str("")
        }
    }
    /// Parses an optionnal chrono::NaiveDateTime structure
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<chrono::NaiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let Ok(s) = String::deserialize(deserializer) {
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S") {
                Ok(Some(dt))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}
