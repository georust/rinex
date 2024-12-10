use crate::{
    context::{Error, ProductType, QcContext, UserBlobData, UserData, UserDataKey},
    prelude::{Merge, Rinex},
};

use std::path::Path;

impl UserBlobData {
    /// Reference to internal [Rinex] data.
    pub fn as_rinex(&self) -> Option<&Rinex> {
        match self {
            Self::Rinex(r) => Some(r),
            _ => None,
        }
    }

    /// Returns mutable reference to inner RINEX data.
    pub fn as_mut_rinex(&mut self) -> Option<&mut Rinex> {
        match self {
            Self::Rinex(r) => Some(r),
            _ => None,
        }
    }
}

impl QcContext {
    /// Load a single [Rinex] into [QcContext].
    /// File revision must be supported, file must be correctly formatted
    /// for this operation to be effective.
    pub fn load_rinex<P: AsRef<Path>>(&mut self, path: P, rinex: Rinex) -> Result<(), Error> {
        let path = path.as_ref();

        let product_type = ProductType::from(rinex.header.rinex_type);

        // extend context
        if let Some(data) = self.get_rinex_data_mut(product_type) {
            let lhs = data.blob_data.as_mut_rinex().unwrap();
            data.paths.push(path.to_path_buf());
            lhs.merge_mut(&rinex)?;
        } else {
            // insert new entry
            let user = UserData {
                paths: vec![path.to_path_buf()],
                blob_data: UserBlobData::Rinex(rinex),
            };
            self.user_data.insert(key, user);
        }

        Ok(())
    }

    // /// Tries to determine a [UniqueId] for this [Rinex].
    // /// This for example, will return unique GNSS receiver identifier.
    // /// It is [RinexType] dependent.
    // fn unique_rinex_id(rinex: &Rinex) -> Option<UniqueId> {
    //     // in special DORIS case: this is the unique satellite ID
    //     // Otherwise use GNSS receiver (if specified)
    //     if let Some(doris) = &rinex.header.doris {
    //         Some(UniqueId::Satellite(doris.satellite.clone()))
    //     } else if let Some(rcvr) = &rinex.header.rcvr {
    //         Some(UniqueId::Receiver(format!("{}-{}", rcvr.model, rcvr.sn)))
    //     } else {
    //         None
    //     }
    // }

    /// Returns reference to inner [Rinex] for this [RinexType]
    pub fn get_rinex_data(&self, product_id: ProductType) -> Option<&Rinex> {
        self.get_per_product_user_data(product_id)?
            .blob_data
            .as_rinex()
    }

    /// Returns mutable reference to inner [Rinex] for this [ProductType]
    pub fn get_rinex_data_mut(&mut self, product_id: ProductType) -> Option<&mut Rinex> {
        self.get_per_product_user_data_mut(product_id)?
            .blob_data
            .as_mut_rinex()
    }

    /// Returns reference to inner [ProductType::Observation] data
    pub fn observation_data(&self) -> Option<&Rinex> {
        self.get_rinex_data(ProductType::Observation)
    }

    /// Returns reference to inner [ProductType::DORIS] RINEX data
    pub fn doris_data(&self) -> Option<&Rinex> {
        self.get_rinex_data(ProductType::DORIS)
    }

    /// Returns reference to inner [ProductType::BroadcastNavigation] data
    pub fn brdc_navigation_data(&self) -> Option<&Rinex> {
        self.get_rinex_data(ProductType::BroadcastNavigation)
    }

    /// Returns reference to inner [ProductType::MeteoObservation] data
    pub fn meteo_data(&self) -> Option<&Rinex> {
        self.get_rinex_data(ProductType::MeteoObservation)
    }

    /// Returns reference to inner [ProductType::HighPrecisionClock] data
    pub fn clock_data(&self) -> Option<&Rinex> {
        self.get_rinex_data(ProductType::HighPrecisionClock)
    }

    /// Returns reference to inner [ProductType::ANTEX] data
    pub fn antex_data(&self) -> Option<&Rinex> {
        self.get_rinex_data(ProductType::ANTEX)
    }

    /// Returns reference to inner [ProductType::IONEX] data
    pub fn ionex_data(&self) -> Option<&Rinex> {
        self.get_rinex_data(ProductType::IONEX)
    }

    /// Returns mutable reference to inner [ProductType::Observation] data
    pub fn observation_data_mut(&mut self) -> Option<&mut Rinex> {
        self.get_rinex_data_mut(ProductType::Observation)
    }

    /// Returns mutable reference to inner [ProductType::DORIS] RINEX data
    pub fn doris_data_mut(&mut self) -> Option<&mut Rinex> {
        self.get_rinex_data_mut(ProductType::DORIS)
    }
    /// Returns mutable reference to inner [ProductType::BroadcastNavigation] data
    pub fn brdc_navigation_data_mut(&mut self) -> Option<&mut Rinex> {
        self.get_rinex_data_mut(ProductType::BroadcastNavigation)
    }

    /// Returns reference to inner [ProductType::MeteoObservation] data
    pub fn meteo_data_mut(&mut self) -> Option<&mut Rinex> {
        self.get_rinex_data_mut(ProductType::MeteoObservation)
    }

    /// Returns mutable reference to inner [ProductType::HighPrecisionClock] data
    pub fn clock_data_mut(&mut self) -> Option<&mut Rinex> {
        self.get_rinex_data_mut(ProductType::HighPrecisionClock)
    }

    /// Returns mutable reference to inner [ProductType::ANTEX] data
    pub fn antex_data_mut(&mut self) -> Option<&mut Rinex> {
        self.get_rinex_data_mut(ProductType::ANTEX)
    }

    /// Returns mutable reference to inner [ProductType::IONEX] data
    pub fn ionex_data_mut(&mut self) -> Option<&mut Rinex> {
        self.get_rinex_data_mut(ProductType::IONEX)
    }

    /// Returns true if [ProductType::Observation] are present in Self
    pub fn has_observation(&self) -> bool {
        self.observation_data().is_some()
    }

    /// Returns true if [QcContext] contains [ProductType::BroadcastNavigation]
    pub fn has_brdc_navigation(&self) -> bool {
        self.brdc_navigation_data().is_some()
    }

    /// Returns true if [QcContext] contains [ProductType::DORIS]
    pub fn has_doris(&self) -> bool {
        self.doris_data().is_some()
    }

    /// Returns true if [QcContext] contains [ProductType::MeteoObservation]
    pub fn has_meteo(&self) -> bool {
        self.meteo_data().is_some()
    }

    /// Returns true if [QcContext] contains [ProductType::HighPrecisionClock]
    pub fn has_clock(&self) -> bool {
        self.clock_data().is_some()
    }

    /// Returns true if [QcContext] contains [ProductType::IONEX]
    pub fn has_ionex(&self) -> bool {
        self.ionex_data().is_some()
    }
}
