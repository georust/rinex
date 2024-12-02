//! Galileo ephemeris
use crate::{utils::Utils, Error};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GALEphemeris {
    pub sv_prn: u8,
    pub toe_week: u16,
    pub tow: i32,
    pub toe_s: i32,
    pub bgd_e5a_e1_s: f32,
    pub bgd_e5b_e1_s: f32,
    pub iodnav: i32,
    pub clock_drift_rate: f32,
    pub clock_drift: f32,
    pub clock_offset: f32,
    pub delta_n_semi_circles_s: f32,
    pub m0_rad: f64,
    pub e: f64,
    pub sqrt_a: f64,
    pub cic: f32,
    pub crc: f32,
    pub cis: f32,
    pub crs: f32,
    pub cuc: f32,
    pub cus: f32,
    pub omega_0_rad: f64,
    pub omega_rad: f64,
    pub i0_rad: f64,
    pub omega_dot_semi_circles: f32,
    pub idot_semi_circles_s: f32,
    pub sisa: f32,
    pub sv_health: u16,
    pub source: u16,
}

impl GALEphemeris {
    pub(crate) const fn encoding_size() -> usize {
        128
    }
    pub(crate) fn encode(&self, big_endian: bool, buf: &mut [u8]) -> Result<usize, Error> {
        let size = Self::encoding_size();
        if buf.len() < size {
            return Err(Error::NotEnoughBytes);
        }

        buf[0] = self.sv_prn;

        let toe_week = if big_endian {
            self.toe_week.to_be_bytes()
        } else {
            self.toe_week.to_le_bytes()
        };

        buf[1..3].copy_from_slice(&toe_week);

        let tow = if big_endian {
            self.tow.to_be_bytes()
        } else {
            self.tow.to_le_bytes()
        };

        buf[3..7].copy_from_slice(&tow);

        let toe_s = if big_endian {
            self.toe_s.to_be_bytes()
        } else {
            self.toe_s.to_le_bytes()
        };

        buf[7..11].copy_from_slice(&toe_s);

        let bgd_e5a_e1_s = if big_endian {
            self.bgd_e5a_e1_s.to_be_bytes()
        } else {
            self.bgd_e5a_e1_s.to_le_bytes()
        };

        buf[11..15].copy_from_slice(&bgd_e5a_e1_s);

        let bgd_e5b_e1_s = if big_endian {
            self.bgd_e5b_e1_s.to_be_bytes()
        } else {
            self.bgd_e5b_e1_s.to_le_bytes()
        };

        buf[15..19].copy_from_slice(&bgd_e5b_e1_s);

        let iodnav = if big_endian {
            self.iodnav.to_be_bytes()
        } else {
            self.iodnav.to_le_bytes()
        };

        buf[19..23].copy_from_slice(&iodnav);

        let clock_drift_rate = if big_endian {
            self.clock_drift_rate.to_be_bytes()
        } else {
            self.clock_drift_rate.to_le_bytes()
        };

        buf[23..27].copy_from_slice(&clock_drift_rate);

        let clock_drift = if big_endian {
            self.clock_drift.to_be_bytes()
        } else {
            self.clock_drift.to_le_bytes()
        };

        buf[27..31].copy_from_slice(&clock_drift);

        let clock_offset = if big_endian {
            self.clock_offset.to_be_bytes()
        } else {
            self.clock_offset.to_le_bytes()
        };

        buf[31..35].copy_from_slice(&clock_offset);

        let delta_n_semi_circles_s = if big_endian {
            self.delta_n_semi_circles_s.to_be_bytes()
        } else {
            self.delta_n_semi_circles_s.to_le_bytes()
        };

        buf[35..39].copy_from_slice(&delta_n_semi_circles_s);

        let m0_rad = if big_endian {
            self.m0_rad.to_be_bytes()
        } else {
            self.m0_rad.to_le_bytes()
        };

        buf[39..47].copy_from_slice(&m0_rad);

        let e = if big_endian {
            self.e.to_be_bytes()
        } else {
            self.e.to_le_bytes()
        };

        buf[47..55].copy_from_slice(&e);

        let sqrt_a = if big_endian {
            self.sqrt_a.to_be_bytes()
        } else {
            self.sqrt_a.to_le_bytes()
        };

        buf[55..63].copy_from_slice(&sqrt_a);

        let cic = if big_endian {
            self.cic.to_be_bytes()
        } else {
            self.cic.to_le_bytes()
        };

        buf[63..67].copy_from_slice(&cic);

        let crc = if big_endian {
            self.crc.to_be_bytes()
        } else {
            self.crc.to_le_bytes()
        };

        buf[67..71].copy_from_slice(&crc);

        let cis = if big_endian {
            self.cis.to_be_bytes()
        } else {
            self.cis.to_le_bytes()
        };

        buf[71..75].copy_from_slice(&cis);

        let crs = if big_endian {
            self.crs.to_be_bytes()
        } else {
            self.crs.to_le_bytes()
        };

        buf[75..79].copy_from_slice(&crs);

        let cuc = if big_endian {
            self.cuc.to_be_bytes()
        } else {
            self.cuc.to_le_bytes()
        };

        buf[79..83].copy_from_slice(&cuc);

        let cus = if big_endian {
            self.cus.to_be_bytes()
        } else {
            self.cus.to_le_bytes()
        };

        buf[83..87].copy_from_slice(&cus);

        let omega_0_rad = if big_endian {
            self.omega_0_rad.to_be_bytes()
        } else {
            self.omega_0_rad.to_le_bytes()
        };

        buf[87..95].copy_from_slice(&omega_0_rad);

        let omega_rad = if big_endian {
            self.omega_rad.to_be_bytes()
        } else {
            self.omega_rad.to_le_bytes()
        };

        buf[95..103].copy_from_slice(&omega_rad);

        let i0_rad = if big_endian {
            self.i0_rad.to_be_bytes()
        } else {
            self.i0_rad.to_le_bytes()
        };

        buf[103..111].copy_from_slice(&i0_rad);

        let omega_dot_semi_circles = if big_endian {
            self.omega_dot_semi_circles.to_be_bytes()
        } else {
            self.omega_dot_semi_circles.to_le_bytes()
        };

        buf[111..115].copy_from_slice(&omega_dot_semi_circles);

        let idot_semi_circles_s = if big_endian {
            self.idot_semi_circles_s.to_be_bytes()
        } else {
            self.idot_semi_circles_s.to_le_bytes()
        };

        buf[115..119].copy_from_slice(&idot_semi_circles_s);

        let sisa = if big_endian {
            self.sisa.to_be_bytes()
        } else {
            self.sisa.to_le_bytes()
        };

        buf[119..123].copy_from_slice(&sisa);

        let sv_health = if big_endian {
            self.sv_health.to_be_bytes()
        } else {
            self.sv_health.to_le_bytes()
        };

        buf[123..125].copy_from_slice(&sv_health);

        let source = if big_endian {
            self.source.to_be_bytes()
        } else {
            self.source.to_le_bytes()
        };

        buf[125..127].copy_from_slice(&source);

        Ok(Self::encoding_size())
    }

    pub(crate) fn decode(big_endian: bool, buf: &[u8]) -> Result<Self, Error> {
        if buf.len() < Self::encoding_size() {
            return Err(Error::NotEnoughBytes);
        }
        // 1. PRN
        let sv_prn = buf[0];
        // 2. TOE
        let toe_week = Utils::decode_u16(big_endian, &buf[1..3])?;
        // 3. TOW
        let tow = Utils::decode_i32(big_endian, &buf[3..7])?;
        // 4. TOE(s)
        let toe_s = Utils::decode_i32(big_endian, &buf[7..11])?;
        // 4. TGD
        let bgd_e5a_e1_s: f32 = Utils::decode_f32(big_endian, &buf[11..15])?;
        let bgd_e5b_e1_s: f32 = Utils::decode_f32(big_endian, &buf[15..19])?;
        // 5. IODNAV
        let iodnav = Utils::decode_i32(big_endian, &buf[19..23])?;
        // 6. Clock
        let clock_drift_rate = Utils::decode_f32(big_endian, &buf[23..27])?;
        let clock_drift = Utils::decode_f32(big_endian, &buf[27..31])?;
        let clock_offset = Utils::decode_f32(big_endian, &buf[31..35])?;
        // 7: delta_n
        let delta_n_semi_circles_s = Utils::decode_f32(big_endian, &buf[35..39])?;
        // 11: m0
        let m0_rad = Utils::decode_f64(big_endian, &buf[39..47])?;
        // 12: e
        let e = Utils::decode_f64(big_endian, &buf[47..55])?;
        // 13: sqrt_a
        let sqrt_a = Utils::decode_f64(big_endian, &buf[55..63])?;
        // 14: cic
        let cic = Utils::decode_f32(big_endian, &buf[63..67])?;
        // 15: crc
        let crc = Utils::decode_f32(big_endian, &buf[67..71])?;
        // 16: cis
        let cis = Utils::decode_f32(big_endian, &buf[71..75])?;
        // 17: crs
        let crs = Utils::decode_f32(big_endian, &buf[75..79])?;
        // 18: cuc
        let cuc = Utils::decode_f32(big_endian, &buf[79..83])?;
        // 19: cus
        let cus = Utils::decode_f32(big_endian, &buf[83..87])?;
        // 20: omega0
        let omega_0_rad = Utils::decode_f64(big_endian, &buf[87..95])?;
        // 21: omega
        let omega_rad = Utils::decode_f64(big_endian, &buf[95..103])?;
        // 22: i0
        let i0_rad = Utils::decode_f64(big_endian, &buf[103..111])?;
        // 23: omega_dot
        let omega_dot_semi_circles = Utils::decode_f32(big_endian, &buf[111..115])?;
        // 24: idot
        let idot_semi_circles_s = Utils::decode_f32(big_endian, &buf[115..119])?;
        // 25: sisa
        let sisa = Utils::decode_f32(big_endian, &buf[119..123])?;
        // 26: sv_health
        let sv_health = Utils::decode_u16(big_endian, &buf[123..125])?;
        // 27: uint2
        let source = Utils::decode_u16(big_endian, &buf[125..127])?;

        Ok(Self {
            sv_prn,
            toe_week,
            tow,
            toe_s,
            bgd_e5a_e1_s,
            bgd_e5b_e1_s,
            iodnav,
            clock_drift_rate,
            clock_drift,
            clock_offset,
            delta_n_semi_circles_s,
            m0_rad,
            e,
            sqrt_a,
            cic,
            crc,
            cis,
            crs,
            cuc,
            cus,
            omega_0_rad,
            omega_rad,
            i0_rad,
            omega_dot_semi_circles,
            idot_semi_circles_s,
            sisa,
            sv_health,
            source,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn eph_x00_x04_error() {
        let buf = [0; 100];
        assert!(GALEphemeris::decode(true, &buf).is_err());
    }

    #[test]
    fn gal_ephemeris() {
        for big_endian in [true, false] {
            let buf = [0; 128];
            let eph = GALEphemeris::decode(big_endian, &buf).unwrap();

            // test mirror
            let mut target = [0; 100];
            assert!(eph.encode(big_endian, &mut target).is_err());

            let mut target = [0; 128];
            let size = eph.encode(big_endian, &mut target).unwrap();
            assert_eq!(size, 128);
            assert_eq!(buf, target);

            let eph = GALEphemeris {
                sv_prn: 10,
                clock_offset: 123.0,
                clock_drift_rate: 130.0,
                clock_drift: 150.0,
                sqrt_a: 56.0,
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
                toe_week: 112,
                tow: -10,
                toe_s: -32,
                bgd_e5a_e1_s: -3.14,
                bgd_e5b_e1_s: -6.18,
                iodnav: -25,
                delta_n_semi_circles_s: 150.0,
                omega_dot_semi_circles: 160.0,
                idot_semi_circles_s: 5000.0,
                sisa: 1000.0,
                sv_health: 155,
                source: 156,
            };

            let mut target = [0; 128];
            eph.encode(big_endian, &mut target).unwrap();

            let decoded = GALEphemeris::decode(big_endian, &target).unwrap();
            assert_eq!(eph, decoded);
        }
    }
}
