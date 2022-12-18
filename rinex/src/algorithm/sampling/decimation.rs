use crate::Duration;

pub trait Decimation<T> {
    /// Decimates Self by given factor.
    /// For example, if record contains epochs {e_0, e_1, .., e_k, ..., e_n}
    /// and we decimate by 2, we're left with epochs {e_0, e_2, ..., e_k, e_k+2, ..}.
    /// Header sampling interval (if any) is automatically adjusted.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::processing::Decimation;
    /// let mut rnx = Rinex::from_file("../test_resources/OBS/V2/delf0010.21o")
    ///     .unwrap();
    /// assert_eq!(rnx.epochs().len(), 105);
    /// rnx.decim_by_ratio_mut(2); // reduce record size by 2
    /// assert_eq!(rnx.epochs().len(), 53);
    /// ```
    fn decim_by_ratio_mut(&mut self, r: u32);

    /// [Decimation::decim_by_ratio_mut] immutable implementation.
    fn decim_by_ratio(&self, r: u32) -> Self;

    /// Decimates Self by minimum epoch duration.
    /// Successive epochs |e_k+1 - e_k| < interval that do not fit
    /// within this minimal interval are discarded.
    /// Header sampling interval (if any) is automatically adjusted.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::processing::*; // Decimation
    /// let mut rinex = Rinex::from_file("../test_resources/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx")
    ///     .unwrap();
    /// let initial_epochs = rinex.epochs();
    /// rinex.decim_by_interval_mut(Duration::from_seconds(10.0));
    /// assert_eq!(rinex.epochs(), initial_epochs); // unchanged, interval is too small
    /// rinex.decim_by_interval_mut(Duration::from_hours(1.0));
    /// assert_eq!(rinex.epochs().len(), initial_epochs.len()-2); // got rid of 2 epochs (15' and 25')
    /// ```
    fn decim_by_interval_mut(&mut self, interval: Duration);

    /// [Decimation::decim_by_interval_mut] immutable implementation.
    fn decim_by_interval(&self, interval: Duration) -> Self;

    /// Decimates Self so sample rate matches `rhs`
    fn decim_match_mut(&mut self, rhs: &Self);
    /// Copies and decimates Self so sample rate matches `rhs`
    fn decim_match(&self, rhs: &Self) -> Self;
}
