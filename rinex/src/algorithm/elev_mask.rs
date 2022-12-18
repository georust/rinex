
#[derive(Debug, Clone)]
pub struct ElevationMask {
    sign: Sign,
    pub angle: f64,
}

#[derive(Debug, Clone)]
pub enum ElevMaskError {
    UnknownSign,
    ValueParsing,
}

impl ElevationMask {
    /// Returns true if `rhs` fits given elevation mask
    pub fn fits(&self, rhs: f64) -> bool {
        match self.sign {
            Sign::Above => rhs >= self.angle,
            Sign::StrictlyAbove => rhs > self.angle,
            Sign::Below => rhs <= self.angle,
            Sign::StrictlyBelow => rhs < self.angle,
        }
    }
}

impl std::str::FromStr for ElevationMask {
    type Err = ElevMaskError;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let content = content.trim();
        if content.starts_with(">=") {
            let angle = &content[2..];
            if let Ok(angle) = f64::from_str(angle.trim()) {
                Ok(Self {
                    sign: Sign::Above,
                    angle,
                })
            } else {
                Err(ElevMaskError::ValueParsing)
            }
        } else if content.starts_with(">") {
            let angle = &content[1..];
            if let Ok(angle) = f64::from_str(angle.trim()) {
                Ok(Self {
                    sign: Sign::StrictlyAbove,
                    angle,
                })
            } else {
                Err(ElevMaskError::ValueParsing)
            }
        } else if content.starts_with("<=") {
            let angle = &content[2..];
            if let Ok(angle) = f64::from_str(angle.trim()) {
                Ok(Self {
                    sign: Sign::Below,
                    angle,
                })
            } else {
                Err(ElevMaskError::ValueParsing)
            }
        } else if content.starts_with("<") {
            let angle = &content[1..];
            if let Ok(angle) = f64::from_str(angle.trim()) {
                Ok(Self {
                    sign: Sign::StrictlyBelow,
                    angle,
                })
            } else {
                Err(ElevMaskError::ValueParsing)
            }
        } else {
            Err(ElevMaskError::UnknownSign)
        }
    }
}
