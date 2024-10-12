use crate::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct GPSRaw {
    uint1: u8,
    sint4: i32,
    bytes: Vec<u8>,
}

impl Default for GPSRaw {
    fn default() -> Self {
        Self {
            uint1: 0,
            sint4: 0,
            bytes: [0; 72].to_vec(),
        }
    }
}

impl GPSRaw {
    /// Builds new Raw GPS Ephemeris message
    pub fn new() -> Self {
        Self::default()
    }
    pub const fn encoding_size() -> usize {
        1 + 4 + 72
    }
    pub fn decode(big_endian: bool, buf: &[u8]) -> Result<Self, Error> {
        if buf.len() < Self::encoding_size() {
            return Err(Error::NotEnoughBytes);
        }

        let uint1 = buf[0];
        let sint4 = if big_endian {
            i32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]])
        } else {
            i32::from_le_bytes([buf[1], buf[2], buf[3], buf[4]])
        };

        Ok(Self {
            uint1,
            sint4,
            bytes: buf[5..72 - 5].to_vec(),
        })
    }
    pub fn encode(&self, big_endian: bool, buf: &mut [u8]) -> Result<usize, Error> {
        let size = Self::encoding_size();
        if buf.len() < size {
            Err(Error::NotEnoughBytes)
        } else {
            buf[0] = self.uint1;

            let bytes = if big_endian {
                self.sint4.to_be_bytes()
            } else {
                self.sint4.to_le_bytes()
            };

            buf[1..5].copy_from_slice(&bytes);
            buf[5..].copy_from_slice(&self.bytes);
            Ok(size)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GPSEphemeris {
    pub sv_prn: u8,
    pub toe: u16,
    pub tow: i32,
    pub toc: i32,
    pub tgd: f32,
    pub iodc: i32,
    /// Clock offset /bias [s]
    pub clock_offset: f32,
    /// Clock drift [s/s]
    pub clock_drift: f32,
    /// Clock drift rate [s/s²]
    pub clock_drift_rate: f32,
    pub iode: i32,
    /// Delta n in [rad/s].
    pub delta_n_rad_sec: f32,
    /// Mean anomaly at reference time [rad]
    pub m0_rad: f64,
    /// Eccentricity
    pub e: f64,
    /// Square root of semi-major axis [m^1/2]
    pub sqrt_a: f64,
    /// cic perturbation
    pub cic: f32,
    /// crc perturbation
    pub crc: f32,
    /// cis perturbation
    pub cis: f32,
    /// crs perturbation
    pub crs: f32,
    /// cuc perturbation
    pub cuc: f32,
    /// cus perturbation
    pub cus: f32,
    /// longitude of ascending node [rad]
    pub omega_0_rad: f64,
    /// argument of perigee [rad]
    pub omega_rad: f64,
    /// inclination at reference time [rad]
    pub i0_rad: f64,
    /// rate of right ascention [rad/s]
    pub omega_dot_rad_s: f32,
    /// rate of inclination [rad/s]
    pub i_dot_rad_s: f32,
    /// nominal User Range Accuracy (URA) in [m]
    pub ura_m: f32,
    // SV health code
    pub sv_health: u16,
    // uint2
    pub uint2: u16,
}

impl GPSEphemeris {
    pub(crate) const fn encoding_size() -> usize {
        72
    }
    pub fn encode(&self, big_endian: bool, buf: &mut [u8]) -> Result<usize, Error> {
        Ok(0)
    }
    pub fn decode(big_endian: bool, buf: &[u8]) -> Result<Self, Error> {
        Ok(Self::default())
    }
}
