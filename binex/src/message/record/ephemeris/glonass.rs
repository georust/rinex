//! Glonass ephemeris
use crate::{utils::Utils, Error};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GLOEphemeris {
    pub slot: u8,
    pub day: u16,
    pub tod_s: u32,
    pub clock_offset_s: f64,
    pub clock_rel_freq_bias: f64,
    pub t_k_sec: u32,
    pub x_km: f64,
    pub vel_x_km: f64,
    pub acc_x_km: f64,
    pub y_km: f64,
    pub vel_y_km: f64,
    pub acc_y_km: f64,
    pub z_km: f64,
    pub vel_z_km: f64,
    pub acc_z_km: f64,
    pub sv_health: u8,
    pub freq_channel: i8,
    pub age_op_days: u8,
    pub leap_s: u8,
    pub tau_gps_s: f64,
    pub l1_l2_gd: f64,
}

impl GLOEphemeris {
    pub(crate) const fn encoding_size() -> usize {
        135
    }
    pub(crate) fn encode(&self, big_endian: bool, buf: &mut [u8]) -> Result<usize, Error> {
        let size = Self::encoding_size();
        if buf.len() < size {
            return Err(Error::NotEnoughBytes);
        }

        buf[0] = self.slot;

        let day = if big_endian {
            self.day.to_be_bytes()
        } else {
            self.day.to_le_bytes()
        };

        buf[1..3].copy_from_slice(&day);

        let tod_s = if big_endian {
            self.tod_s.to_be_bytes()
        } else {
            self.tod_s.to_le_bytes()
        };

        buf[4..8].copy_from_slice(&tod_s);

        let clock_offset_s = if big_endian {
            self.clock_offset_s.to_be_bytes()
        } else {
            self.clock_offset_s.to_le_bytes()
        };

        buf[9..17].copy_from_slice(&clock_offset_s);

        let clock_rel_freq_bias = if big_endian {
            self.clock_rel_freq_bias.to_be_bytes()
        } else {
            self.clock_rel_freq_bias.to_le_bytes()
        };

        buf[18..26].copy_from_slice(&clock_rel_freq_bias);

        let t_k_sec = if big_endian {
            self.t_k_sec.to_be_bytes()
        } else {
            self.t_k_sec.to_le_bytes()
        };

        buf[27..31].copy_from_slice(&t_k_sec);

        let x_km = if big_endian {
            self.x_km.to_be_bytes()
        } else {
            self.x_km.to_le_bytes()
        };

        buf[32..40].copy_from_slice(&x_km);

        let vel_x_km = if big_endian {
            self.vel_x_km.to_be_bytes()
        } else {
            self.vel_x_km.to_le_bytes()
        };

        buf[41..49].copy_from_slice(&vel_x_km);

        let acc_x_km = if big_endian {
            self.acc_x_km.to_be_bytes()
        } else {
            self.acc_x_km.to_le_bytes()
        };

        buf[50..58].copy_from_slice(&acc_x_km);

        let y_km = if big_endian {
            self.y_km.to_be_bytes()
        } else {
            self.y_km.to_le_bytes()
        };

        buf[59..67].copy_from_slice(&y_km);

        let vel_y_km = if big_endian {
            self.vel_y_km.to_be_bytes()
        } else {
            self.vel_y_km.to_le_bytes()
        };

        buf[68..76].copy_from_slice(&vel_y_km);

        let acc_y_km = if big_endian {
            self.acc_y_km.to_be_bytes()
        } else {
            self.acc_y_km.to_le_bytes()
        };

        buf[77..85].copy_from_slice(&acc_y_km);

        let z_km = if big_endian {
            self.z_km.to_be_bytes()
        } else {
            self.z_km.to_le_bytes()
        };

        buf[86..94].copy_from_slice(&z_km);

        let vel_z_km = if big_endian {
            self.vel_z_km.to_be_bytes()
        } else {
            self.vel_z_km.to_le_bytes()
        };

        buf[95..103].copy_from_slice(&vel_z_km);

        let acc_z_km = if big_endian {
            self.acc_z_km.to_be_bytes()
        } else {
            self.acc_z_km.to_le_bytes()
        };

        buf[104..112].copy_from_slice(&acc_z_km);

        buf[113] = self.sv_health;
        buf[114] = self.freq_channel as u8;
        buf[115] = self.age_op_days;
        buf[116] = self.leap_s;

        let tau_gps_s = if big_endian {
            self.tau_gps_s.to_be_bytes()
        } else {
            self.tau_gps_s.to_le_bytes()
        };

        buf[117..125].copy_from_slice(&tau_gps_s);

        let l1_l2_gd = if big_endian {
            self.l1_l2_gd.to_be_bytes()
        } else {
            self.l1_l2_gd.to_le_bytes()
        };

        buf[126..134].copy_from_slice(&l1_l2_gd);

        Ok(135)
    }

    pub(crate) fn decode(big_endian: bool, buf: &[u8]) -> Result<Self, Error> {
        if buf.len() < Self::encoding_size() {
            return Err(Error::NotEnoughBytes);
        }
        // 1. PRN
        let slot = buf[0];
        // 2. DAY
        let day = Utils::decode_u16(big_endian, &buf[1..3])?;
        // 3. TOD
        let tod_s = Utils::decode_u32(big_endian, &buf[4..8])?;
        // 4. Clock
        let clock_offset_s = Utils::decode_f64(big_endian, &buf[9..17])?;
        // 4. Clock
        let clock_rel_freq_bias = Utils::decode_f64(big_endian, &buf[18..26])?;
        // 5. t_k
        let t_k_sec = Utils::decode_u32(big_endian, &buf[27..31])?;
        // 6. x
        let x_km = Utils::decode_f64(big_endian, &buf[32..40])?;
        let vel_x_km = Utils::decode_f64(big_endian, &buf[41..49])?;
        let acc_x_km = Utils::decode_f64(big_endian, &buf[50..58])?;
        // 7. y
        let y_km = Utils::decode_f64(big_endian, &buf[59..67])?;
        let vel_y_km = Utils::decode_f64(big_endian, &buf[68..76])?;
        let acc_y_km = Utils::decode_f64(big_endian, &buf[77..85])?;
        // 8. z
        let z_km = Utils::decode_f64(big_endian, &buf[86..94])?;
        let vel_z_km = Utils::decode_f64(big_endian, &buf[95..103])?;
        let acc_z_km = Utils::decode_f64(big_endian, &buf[104..112])?;
        // 9. bits
        let sv_health = buf[113];
        let freq_channel = buf[114] as i8;
        let age_op_days = buf[115];
        let leap_s = buf[116];

        // 10 tau_gps_s
        let tau_gps_s = Utils::decode_f64(big_endian, &buf[117..125])?;
        // 11: l1/l2 gd
        let l1_l2_gd = Utils::decode_f64(big_endian, &buf[126..134])?;

        Ok(Self {
            slot,
            day,
            tod_s,
            clock_offset_s,
            clock_rel_freq_bias,
            t_k_sec,
            x_km,
            vel_x_km,
            acc_x_km,
            y_km,
            vel_y_km,
            acc_y_km,
            z_km,
            vel_z_km,
            acc_z_km,
            sv_health,
            freq_channel,
            age_op_days,
            leap_s,
            tau_gps_s,
            l1_l2_gd,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn eph_x00_x02_error() {
        let buf = [0; 100];
        assert!(GLOEphemeris::decode(true, &buf).is_err());
    }

    #[test]
    fn glo_ephemeris() {
        for big_endian in [true, false] {
            let buf = [0; 135];
            let eph = GLOEphemeris::decode(big_endian, &buf).unwrap();

            // test mirror
            let mut encoded = [0; 100];
            assert!(eph.encode(big_endian, &mut encoded).is_err());

            let mut encoded = [0; 135];
            let size = eph.encode(big_endian, &mut encoded).unwrap();
            assert_eq!(size, 135);
            assert_eq!(buf, encoded);

            let eph = GLOEphemeris {
                t_k_sec: 0,
                slot: 1,
                day: 2,
                tod_s: 3,
                clock_offset_s: 1.0,
                clock_rel_freq_bias: 2.0,
                x_km: 3.0,
                vel_x_km: 4.0,
                acc_x_km: 4.0,
                y_km: 5.0,
                vel_y_km: 6.0,
                acc_y_km: 7.0,
                z_km: 8.0,
                vel_z_km: 9.0,
                acc_z_km: 10.0,
                sv_health: 100,
                freq_channel: -20,
                age_op_days: 123,
                leap_s: 124,
                tau_gps_s: 3.14,
                l1_l2_gd: 6.28,
            };

            let mut encoded = [0; 135];
            eph.encode(big_endian, &mut encoded).unwrap();

            let decoded = GLOEphemeris::decode(big_endian, &encoded).unwrap();
            assert_eq!(eph, decoded);
        }
    }
}
