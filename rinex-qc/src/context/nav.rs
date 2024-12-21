use rinex::prelude::Rinex;

use crate::context::{QcContext, QcError};

use qc_traits::Merge;

impl QcContext {
    pub fn has_navigation_data(&self) -> bool {
        self.nav_dataset.is_some()
    }

    /// Loads a new Navigation [Rinex] into this [QcContext]
    pub(crate) fn load_navigation_rinex(&mut self, data: Rinex) -> Result<(), QcError> {
        // proceed to stacking
        if let Some(nav) = &mut self.nav_dataset {
            nav.merge_mut(&data)?;
        } else {
            self.nav_dataset = Some(data);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use crate::{cfg::QcConfig, context::QcContext};

    #[test]
    fn navigation_indexing() {
        let path = format!(
            "{}/../test_resources/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx",
            env!("CARGO_MANIFEST_DIR")
        );

        // Default indexing
        let cfg = QcConfig::default();

        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_file(&path).unwrap();
        assert!(ctx.has_navigation_data());
    }

    #[test]
    fn navigation_stacking() {
        let path_1 = format!(
            "{}/../test_resources/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx",
            env!("CARGO_MANIFEST_DIR")
        );

        let path_2 = format!(
            "{}/../test_resources/NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx",
            env!("CARGO_MANIFEST_DIR")
        );

        let cfg = QcConfig::default();
        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_file(&path_1).unwrap();
        assert!(ctx.has_navigation_data());

        ctx.load_file(&path_1).unwrap();
        assert!(ctx.has_navigation_data());

        let cfg = QcConfig::default();
        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_file(&path_1).unwrap();
        assert!(ctx.has_navigation_data());

        ctx.load_file(&path_2).unwrap();
        assert!(ctx.has_navigation_data());
    }
}
