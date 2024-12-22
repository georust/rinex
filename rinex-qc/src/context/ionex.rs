use rinex::prelude::Rinex;

use crate::context::{QcContext, QcError};

use qc_traits::Merge;

impl QcContext {
    pub fn has_ionosphere_maps(&self) -> bool {
        self.ionex_dataset.is_some()
    }

    /// Loads a new IONex [Rinex] into this [QcContext]
    pub(crate) fn load_ionex(&mut self, data: Rinex) -> Result<(), QcError> {
        // proceed to stacking
        if let Some(rinex) = &mut self.ionex_dataset {
            rinex.merge_mut(&data)?;
        } else {
            self.ionex_dataset = Some(data);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use crate::{cfg::QcConfig, context::QcContext};

    #[test]
    #[cfg(feature = "flate2")]
    fn ionex_indexing() {
        let path = format!(
            "{}/../test_resources/IONEX/V1/CKMG0020.22I.gz",
            env!("CARGO_MANIFEST_DIR")
        );

        // Default indexing
        let cfg = QcConfig::default();

        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_gzip_file(&path).unwrap();
        assert!(ctx.has_ionosphere_maps());
    }

    #[test]
    #[cfg(feature = "flate2")]
    fn ionex_stacking() {
        let path = format!(
            "{}/../test_resources/IONEX/V1/CKMG0020.22I.gz",
            env!("CARGO_MANIFEST_DIR")
        );

        let path_2 = format!(
            "{}/../test_resources/IONEX/V1/CKMG0090.21I.gz",
            env!("CARGO_MANIFEST_DIR")
        );

        let cfg = QcConfig::default();
        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_gzip_file(&path).unwrap();
        assert!(ctx.has_ionosphere_maps());

        ctx.load_gzip_file(&path_2).unwrap();
        assert!(ctx.has_ionosphere_maps());
    }
}
