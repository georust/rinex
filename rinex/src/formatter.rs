#[cfg(feature = "serde")]
pub mod opt_point3d {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::str::FromStr;

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
                        return Ok(Some(rust_3d::Point3D::new(x, y, z)));
                    }
                }
            }
        }
        Ok(None)
    }
}
