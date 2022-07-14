#[cfg(feature = "with-serde")]
pub mod point3d {
    pub fn serialize<S>(point3d: &Option<rust_3d::Point3D>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let p = point3d.as_ref().unwrap_or(
            &rust_3d::Point3D {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 0.0_f64,
            }
        );
        let s = format!("{},{},{}",p.x,p.y,p.z); 
        serializer.serialize_str(&s)
    }
}


#[cfg(feature = "with-serde")]
pub mod datetime {
    use serde::{Serializer};
    pub fn serialize<S>(datetime: &chrono::NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", datetime.format("%Y-%m-%d %H:%M:%S"));
        serializer.serialize_str(&s)
    }

    /*pub fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>, 
    {
        let s = String::deserialize(deserializer)?;
        chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")?
    }*/
}
