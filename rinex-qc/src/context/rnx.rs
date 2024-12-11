use crate::context::{user_rover::UserRoverData, MetaData, QcContext};
use anise::prelude::Orbit;
use rinex::prelude::{Rinex, RinexType};

impl QcContext {
    /// Loads a new Observation [Rinex] into this [QcContext]
    fn load_observation_rinex(&mut self, meta: MetaData, data: Rinex) {
        self.user_rover_observations = Some(UserRoverData {
            meta,
            ground_position: if let Some(ground_position) = &data.header.ground_position {
                let (ecef_x_m, ecef_y_m, ecef_z_m) = ground_position.to_ecef_wgs84();
                if let Some(t) = data.first_epoch() {
                    let orbit = Orbit::from_position(
                        ecef_x_m / 1000.0,
                        ecef_y_m / 1000.0,
                        ecef_z_m / 1000.0,
                        t,
                        self.earth_cef,
                    );
                    Some(orbit)
                } else {
                    None
                }
            } else {
                None
            },
            data,
        });
    }

    /// Load a single [Rinex] into [QcContext].
    /// File revision must be supported, file must be correctly formatted
    /// for this operation to be effective.
    pub fn load_rinex(&mut self, meta: MetaData, rinex: Rinex) {
        // Classification is rinex type dependent
        let rinex_type = rinex.header.rinex_type;
        match rinex_type {
            RinexType::DORIS => {
                panic!("DORIS not supported!");
            },
            RinexType::AntennaData => {
                panic!("ANTEX not handled yet!");
            },
            RinexType::ClockData => {
                self.clk_context.load_rinex(&meta, rinex);
            },
            RinexType::IonosphereMaps => {
                self.iono_context.load_rinex(&meta, rinex);
            },
            RinexType::MeteoData => {
                self.meteo_context.load_rinex(&meta, rinex);
            },
            RinexType::NavigationData => {
                self.sky_context.load_rinex(&meta, rinex);
            },
            RinexType::ObservationData => {
                self.load_observation_rinex(meta, rinex);
            },
        }
    }
}
