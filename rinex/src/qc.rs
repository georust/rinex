use crate::prelude::{Epoch, Merge, MergeError, Rinex, Split};

impl Merge for Rinex {
    /// [Merge] [Rinex] (rhs) into Self, forming a new dataset.
    /// When merging two RINEX together, the file formats must be identical.
    /// [Header] is adapted yet Self's attribute are preserved: only new information is provided.
    /// ```
    /// use rinex::prelude::*;
    /// let rnx_a = Rinex::from_file("../test_resources/OBS/V2/delf0010.21o")
    ///     .unwrap();
    /// let rnx_b = Rinex::from_file("../test_resources/NAV/V2/amel0010.21g")
    ///     .unwrap();
    /// let merged = rnx_a.merge(&rnx_b);
    /// // When merging, RINEX format must match
    /// assert_eq!(merged.is_ok(), false);
    /// let rnx_b = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    /// let merged = rnx_a.merge(&rnx_b);
    /// assert_eq!(merged.is_ok(), true);
    /// let merged = merged.unwrap();
    /// // when merging, Self's attributes are always prefered.
    /// // Results have most delf0010.21o attributes
    /// // Only new attributes, that 'DUTH0630.22O' would introduced are stored
    /// assert_eq!(merged.header.version.major, 2);
    /// assert_eq!(merged.header.version.minor, 11);
    /// assert_eq!(merged.header.program, "teqc  2019Feb25");
    /// // Resulting RINEX will therefore follow RINEX2 specifications
    /// assert!(merged.to_file("merge.rnx").is_ok(), "failed to merge file");
    /// ```
    fn merge(&self, rhs: &Self) -> Result<Self, MergeError> {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
    /// Merges `rhs` into `Self` in place
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), MergeError> {
        self.header.merge_mut(&rhs.header)?;
        if !self.is_antex() {
            if self.epoch().count() == 0 {
                // lhs is empty : overwrite
                self.record = rhs.record.clone();
            } else if rhs.epoch().count() != 0 {
                // real merge
                self.record.merge_mut(&rhs.record)?;
            }
        } else {
            // real merge
            self.record.merge_mut(&rhs.record)?;
        }
        Ok(())
    }
}

impl Split for Rinex {
    fn split(&self, epoch: Epoch) -> (Self, Self) {
        let (r0, r1) = self.record.split(epoch);
        (
            Self {
                record: r0,
                header: self.header.clone(),
                comments: self.comments.clone(), // TODO: rework
                prod_attr: self.prod_attr.clone(),
            },
            Self {
                record: r1,
                header: self.header.clone(),
                comments: self.comments.clone(), // TODO: rework
                prod_attr: self.prod_attr.clone(),
            },
        )
    }
    fn split_mut(&mut self, epoch: Epoch) -> Self {
        let r = self.record.split_mut(epoch);
        Self {
            record: r,
            header: self.header.clone(),
            coments: self.comments.clone(),    // TODO: rework
            prod_attr: self.prod_attr.clone(), // TODO: crosscheck
        }
    }
    fn split_even_dt(&self, dt: hifitime::Duration) -> Vec<Self> {
        Default::default()
    }
}
