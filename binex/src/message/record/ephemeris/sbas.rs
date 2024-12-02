//! SBAS ephemeris
use crate::{utils::Utils, Error};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SBASEphemeris {
    pub sbas_prn: u8,
    pub toe: u16,
    pub tow: i32,
    /// Clock offset /bias [s]
    pub clock_offset: f64,
    /// Clock drift [s/s]
    pub clock_drift: f64,
    pub x_km: f64,
    pub vel_x_km: f64,
    pub acc_x_km: f64,
    pub y_km: f64,
    pub vel_y_km: f64,
    pub acc_y_km: f64,
    pub z_km: f64,
    pub vel_z_km: f64,
    pub acc_z_km: f64,
    pub uint1: u8,
    pub ura: u8,
    pub iodn: u8,
}

impl SBASEphemeris {
    pub(crate) const fn encoding_size() -> usize {
        98
    }
    pub(crate) fn encode(&self, big_endian: bool, buf: &mut [u8]) -> Result<usize, Error> {
        let size = Self::encoding_size();
        if buf.len() < size {
            return Err(Error::NotEnoughBytes);
        }

        buf[0] = self.sbas_prn;

        let toe = if big_endian {
            self.toe.to_be_bytes()
        } else {
            self.toe.to_le_bytes()
        };

        buf[1..3].copy_from_slice(&toe);

        let tow = if big_endian {
            self.tow.to_be_bytes()
        } else {
            self.tow.to_le_bytes()
        };

        buf[3..7].copy_from_slice(&tow);

        let clock_offset = if big_endian {
            self.clock_offset.to_be_bytes()
        } else {
            self.clock_offset.to_le_bytes()
        };

        buf[7..15].copy_from_slice(&clock_offset);

        let clock_drift = if big_endian {
            self.clock_drift.to_be_bytes()
        } else {
            self.clock_drift.to_le_bytes()
        };

        buf[15..23].copy_from_slice(&clock_drift);

        let x_km = if big_endian {
            self.x_km.to_be_bytes()
        } else {
            self.x_km.to_le_bytes()
        };

        buf[23..31].copy_from_slice(&x_km);

        let vel_x_km = if big_endian {
            self.vel_x_km.to_be_bytes()
        } else {
            self.vel_x_km.to_le_bytes()
        };

        buf[31..39].copy_from_slice(&vel_x_km);

        let acc_x_km = if big_endian {
            self.acc_x_km.to_be_bytes()
        } else {
            self.acc_x_km.to_le_bytes()
        };

        buf[39..47].copy_from_slice(&acc_x_km);

        let y_km = if big_endian {
            self.y_km.to_be_bytes()
        } else {
            self.y_km.to_le_bytes()
        };

        buf[47..55].copy_from_slice(&y_km);

        let vel_y_km = if big_endian {
            self.vel_y_km.to_be_bytes()
        } else {
            self.vel_y_km.to_le_bytes()
        };

        buf[55..63].copy_from_slice(&vel_y_km);

        let acc_y_km = if big_endian {
            self.acc_y_km.to_be_bytes()
        } else {
            self.acc_y_km.to_le_bytes()
        };

        buf[63..71].copy_from_slice(&acc_y_km);

        let z_km = if big_endian {
            self.z_km.to_be_bytes()
        } else {
            self.z_km.to_le_bytes()
        };

        buf[71..79].copy_from_slice(&z_km);

        let vel_z_km = if big_endian {
            self.vel_z_km.to_be_bytes()
        } else {
            self.vel_z_km.to_le_bytes()
        };

        buf[79..87].copy_from_slice(&vel_z_km);

        let acc_z_km = if big_endian {
            self.acc_z_km.to_be_bytes()
        } else {
            self.acc_z_km.to_le_bytes()
        };

        buf[87..95].copy_from_slice(&acc_z_km);

        buf[95] = self.uint1;
        buf[96] = self.ura;
        buf[97] = self.iodn;

        Ok(Self::encoding_size())
    }
    pub(crate) fn decode(big_endian: bool, buf: &[u8]) -> Result<Self, Error> {
        if buf.len() < Self::encoding_size() {
            return Err(Error::NotEnoughBytes);
        }
        // 1. PRN
        let sbas_prn = buf[0];
        // 2. TOE
        let toe = Utils::decode_u16(big_endian, &buf[1..3])?;
        // 3. TOW
        let tow = Utils::decode_i32(big_endian, &buf[3..7])?;
        // 4. Clock
        let clock_offset = Utils::decode_f64(big_endian, &buf[7..15])?;
        let clock_drift = Utils::decode_f64(big_endian, &buf[15..23])?;
        // 5. x
        let x_km = Utils::decode_f64(big_endian, &buf[23..31])?;
        let vel_x_km = Utils::decode_f64(big_endian, &buf[31..39])?;
        let acc_x_km = Utils::decode_f64(big_endian, &buf[39..47])?;
        // 6: y
        let y_km = Utils::decode_f64(big_endian, &buf[47..55])?;
        let vel_y_km = Utils::decode_f64(big_endian, &buf[55..63])?;
        let acc_y_km = Utils::decode_f64(big_endian, &buf[63..71])?;
        // 6: z
        let z_km = Utils::decode_f64(big_endian, &buf[71..79])?;
        let vel_z_km = Utils::decode_f64(big_endian, &buf[79..87])?;
        let acc_z_km = Utils::decode_f64(big_endian, &buf[87..95])?;
        // 7: bits
        let uint1 = buf[95];
        let ura = buf[96];
        let iodn = buf[97];

        Ok(Self {
            sbas_prn,
            toe,
            tow,
            clock_offset,
            clock_drift,
            x_km,
            vel_x_km,
            acc_x_km,
            y_km,
            vel_y_km,
            acc_y_km,
            z_km,
            vel_z_km,
            acc_z_km,
            uint1,
            ura,
            iodn,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn eph_x00_x03_error() {
        let buf = [0; 64];
        assert!(SBASEphemeris::decode(true, &buf).is_err());
    }

    #[test]
    fn sbas_ephemeris() {
        for big_endian in [true, false] {
            let buf = [0; 100];
            let eph = SBASEphemeris::decode(big_endian, &buf).unwrap();

            // test mirror
            let mut target = [0; 64];
            assert!(eph.encode(big_endian, &mut target).is_err());

            let mut target = [0; 100];
            let size = eph.encode(big_endian, &mut target).unwrap();
            assert_eq!(size, 98);
            assert_eq!(buf, target);

            let eph = SBASEphemeris {
                sbas_prn: 10,
                toe: 11,
                tow: 12,
                clock_drift: 0.1,
                clock_offset: 0.2,
                x_km: 1.4,
                vel_x_km: 1.5,
                acc_x_km: 1.6,
                y_km: 2.4,
                vel_y_km: 2.5,
                acc_y_km: 2.6,
                z_km: 3.1,
                vel_z_km: 3.2,
                acc_z_km: 3.3,
                uint1: 4,
                ura: 5,
                iodn: 6,
            };

            let mut target = [0; 100];
            eph.encode(big_endian, &mut target).unwrap();

            let decoded = SBASEphemeris::decode(big_endian, &target).unwrap();
            assert_eq!(eph, decoded);
        }
    }
}
