use std::f32::consts::PI as Pi32;

use crate::{utils::Utils, Error};

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
    pub delta_n_rad_s: f32,
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
        153
    }
    pub fn encode(&self, big_endian: bool, buf: &mut [u8]) -> Result<usize, Error> {
        let size = Self::encoding_size();
        if buf.len() < size {
            return Err(Error::NotEnoughBytes);
        }

        buf[0] = self.sv_prn;

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

        buf[4..8].copy_from_slice(&tow);

        let toc = if big_endian {
            self.toc.to_be_bytes()
        } else {
            self.toc.to_le_bytes()
        };

        buf[9..13].copy_from_slice(&toc);

        let tgd = if big_endian {
            self.tgd.to_be_bytes()
        } else {
            self.tgd.to_le_bytes()
        };

        buf[14..18].copy_from_slice(&tgd);

        let iodc = if big_endian {
            self.iodc.to_be_bytes()
        } else {
            self.iodc.to_le_bytes()
        };

        buf[19..23].copy_from_slice(&iodc);

        let af2 = if big_endian {
            self.clock_drift_rate.to_be_bytes()
        } else {
            self.clock_drift_rate.to_le_bytes()
        };

        buf[24..28].copy_from_slice(&af2);

        let af1 = if big_endian {
            self.clock_drift.to_be_bytes()
        } else {
            self.clock_drift.to_le_bytes()
        };

        buf[29..33].copy_from_slice(&af1);

        let af0 = if big_endian {
            self.clock_offset.to_be_bytes()
        } else {
            self.clock_offset.to_le_bytes()
        };

        buf[34..38].copy_from_slice(&af0);

        let iode = if big_endian {
            self.iode.to_be_bytes()
        } else {
            self.iode.to_le_bytes()
        };

        buf[39..43].copy_from_slice(&iode);

        let delta_n = if big_endian {
            (self.delta_n_rad_s / Pi32).to_be_bytes()
        } else {
            (self.delta_n_rad_s / Pi32).to_le_bytes()
        };

        buf[44..48].copy_from_slice(&delta_n);

        let m0 = if big_endian {
            self.m0_rad.to_be_bytes()
        } else {
            self.m0_rad.to_le_bytes()
        };

        buf[49..57].copy_from_slice(&m0);

        let e = if big_endian {
            self.e.to_be_bytes()
        } else {
            self.e.to_le_bytes()
        };

        buf[58..66].copy_from_slice(&e);

        let sqrt_a = if big_endian {
            self.sqrt_a.to_be_bytes()
        } else {
            self.sqrt_a.to_le_bytes()
        };

        buf[67..75].copy_from_slice(&sqrt_a);

        let cic = if big_endian {
            self.cic.to_be_bytes()
        } else {
            self.cic.to_le_bytes()
        };

        buf[76..80].copy_from_slice(&cic);

        let crc = if big_endian {
            self.crc.to_be_bytes()
        } else {
            self.crc.to_le_bytes()
        };

        buf[81..85].copy_from_slice(&crc);

        let cis = if big_endian {
            self.cis.to_be_bytes()
        } else {
            self.cis.to_le_bytes()
        };

        buf[86..90].copy_from_slice(&cis);

        let crs = if big_endian {
            self.crs.to_be_bytes()
        } else {
            self.crs.to_le_bytes()
        };

        buf[91..95].copy_from_slice(&crs);

        let cuc = if big_endian {
            self.cuc.to_be_bytes()
        } else {
            self.cuc.to_le_bytes()
        };

        buf[96..100].copy_from_slice(&cuc);

        let cus = if big_endian {
            self.cus.to_be_bytes()
        } else {
            self.cus.to_le_bytes()
        };

        buf[101..105].copy_from_slice(&cus);

        let omega_0_rad = if big_endian {
            self.omega_0_rad.to_be_bytes()
        } else {
            self.omega_0_rad.to_le_bytes()
        };

        buf[106..114].copy_from_slice(&omega_0_rad);

        let omega_rad = if big_endian {
            self.omega_rad.to_be_bytes()
        } else {
            self.omega_rad.to_le_bytes()
        };

        buf[115..123].copy_from_slice(&omega_rad);

        let i0_rad = if big_endian {
            self.i0_rad.to_be_bytes()
        } else {
            self.i0_rad.to_le_bytes()
        };

        buf[124..132].copy_from_slice(&i0_rad);

        let omega_dot_rad_s = if big_endian {
            (self.omega_dot_rad_s / Pi32).to_be_bytes()
        } else {
            (self.omega_dot_rad_s / Pi32).to_le_bytes()
        };

        buf[133..137].copy_from_slice(&omega_dot_rad_s);

        let i_dot_rad_s = if big_endian {
            (self.i_dot_rad_s / Pi32).to_be_bytes()
        } else {
            (self.i_dot_rad_s / Pi32).to_le_bytes()
        };

        buf[138..142].copy_from_slice(&i_dot_rad_s);

        let ura_m = if big_endian {
            (self.ura_m / 0.1).to_be_bytes()
        } else {
            (self.ura_m / 0.1).to_le_bytes()
        };

        buf[143..147].copy_from_slice(&ura_m);

        let sv_health = if big_endian {
            self.sv_health.to_be_bytes()
        } else {
            self.sv_health.to_le_bytes()
        };

        buf[148..150].copy_from_slice(&sv_health);

        let uint2 = if big_endian {
            self.uint2.to_be_bytes()
        } else {
            self.uint2.to_le_bytes()
        };

        buf[151..153].copy_from_slice(&uint2);

        Ok(size)
    }
    pub fn decode(big_endian: bool, buf: &[u8]) -> Result<Self, Error> {
        if buf.len() < Self::encoding_size() {
            return Err(Error::NotEnoughBytes);
        }
        // 1. PRN
        let sv_prn = buf[0];
        // 2. TOE
        let toe = Utils::decode_u16(big_endian, &buf[1..3])?;
        // 3. TOW
        let tow = Utils::decode_i32(big_endian, &buf[4..8])?;
        // 4. TOC
        let toc = Utils::decode_i32(big_endian, &buf[9..13])?;
        // 4. TGD
        let tgd = Utils::decode_f32(big_endian, &buf[14..18])?;
        // 5. IODC
        let iodc = Utils::decode_i32(big_endian, &buf[19..23])?;
        // 6. Af2
        let af2 = Utils::decode_f32(big_endian, &buf[24..28])?;
        // 7. Af1
        let af1 = Utils::decode_f32(big_endian, &buf[29..33])?;
        // 8. Af0
        let af0 = Utils::decode_f32(big_endian, &buf[34..38])?;
        // 9: IODE
        let iode = Utils::decode_i32(big_endian, &buf[39..43])?;
        // 10: delta_n
        let delta_n_rad_s = Utils::decode_f32(big_endian, &buf[44..48])? * Pi32;
        // 11: m0
        let m0_rad = Utils::decode_f64(big_endian, &buf[49..57])?;
        // 12: e
        let e = Utils::decode_f64(big_endian, &buf[58..66])?;
        // 13: sqrt_a
        let sqrt_a = Utils::decode_f64(big_endian, &buf[67..75])?;
        // 14: cic
        let cic = Utils::decode_f32(big_endian, &buf[76..80])?;
        // 15: crc
        let crc = Utils::decode_f32(big_endian, &buf[81..85])?;
        // 16: cis
        let cis = Utils::decode_f32(big_endian, &buf[86..90])?;
        // 17: crs
        let crs = Utils::decode_f32(big_endian, &buf[91..95])?;
        // 18: cuc
        let cuc = Utils::decode_f32(big_endian, &buf[96..100])?;
        // 19: cus
        let cus = Utils::decode_f32(big_endian, &buf[101..105])?;
        // 20: omega0
        let omega_0_rad = Utils::decode_f64(big_endian, &buf[106..114])?;
        // 21: omega
        let omega_rad = Utils::decode_f64(big_endian, &buf[115..123])?;
        // 22: i0
        let i0_rad = Utils::decode_f64(big_endian, &buf[124..132])?;
        // 23: omega_dot
        let omega_dot_rad_s = Utils::decode_f32(big_endian, &buf[133..137])? * Pi32;
        // 24: idot
        let i_dot_rad_s = Utils::decode_f32(big_endian, &buf[138..142])? * Pi32;
        // 25: ura
        let ura_m = Utils::decode_f32(big_endian, &buf[143..147])? * 0.1;
        // 26: sv_health
        let sv_health = Utils::decode_u16(big_endian, &buf[148..150])?;
        // 27: uint2
        let uint2 = Utils::decode_u16(big_endian, &buf[151..153])?;

        Ok(Self {
            sv_prn,
            toe,
            tow,
            toc,
            tgd,
            iodc,
            iode,
            clock_offset: af0,
            clock_drift: af1,
            clock_drift_rate: af2,
            delta_n_rad_s,
            m0_rad,
            e,
            sqrt_a,
            cic,
            crc,
            cis,
            crs,
            cuc,
            cus,
            omega_rad,
            omega_0_rad,
            i0_rad,
            i_dot_rad_s,
            omega_dot_rad_s,
            ura_m,
            sv_health,
            uint2,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn eph_x00_x01_error() {
        let buf = [0; 100];
        assert!(GPSEphemeris::decode(true, &buf).is_err());
    }

    #[test]
    fn gps_ephemeris() {
        let buf = [0; 153];

        let eph = GPSEphemeris::decode(true, &buf).unwrap();

        // test mirror
        let mut target = [0; 100];
        assert!(eph.encode(true, &mut target).is_err());

        let mut target = [0; 153];
        let size = eph.encode(true, &mut target).unwrap();
        assert_eq!(size, 153);
        assert_eq!(buf, target);

        let eph = GPSEphemeris {
            sv_prn: 10,
            toe: 1000,
            tow: 120,
            toc: 130,
            tgd: 10.0,
            iodc: 24,
            clock_offset: 123.0,
            clock_drift_rate: 130.0,
            clock_drift: 150.0,
            sqrt_a: 56.0,
            iode: -2000,
            delta_n_rad_s: 12.0,
            m0_rad: 0.1,
            e: 0.2,
            cic: 0.3,
            crc: 0.4,
            cis: 0.5,
            crs: 0.6,
            cuc: 0.7,
            cus: 0.8,
            omega_0_rad: 0.9,
            omega_rad: 59.0,
            i0_rad: 61.0,
            omega_dot_rad_s: 62.0,
            i_dot_rad_s: 74.0,
            ura_m: 75.0,
            sv_health: 16,
            uint2: 17,
        };

        let mut target = [0; 153];
        eph.encode(true, &mut target).unwrap();

        let decoded = GPSEphemeris::decode(true, &target).unwrap();

        assert_eq!(eph, decoded);
    }
}
