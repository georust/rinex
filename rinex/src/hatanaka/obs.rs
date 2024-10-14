//! Observation Compression kernel

use crate::hatanaka::{
    NumDiff, TextDiff,
};

pub struct ObsDiff<const M: usize> {
    pub obs_ptr: usize,
    pub data_diff: NumDiff<M>,
    pub snr_diff: TextDiff,
    pub lli_diff: TextDiff,
}

impl<const M: usize> ObsDiff<M> {
    pub fn new(obsptr: usize, obsdata: i64, snr: &str, lli: &str) -> Self {
        Self {
            obs_ptr: obsptr,
            data_diff: NumDiff::<M>::new(obsdata),
            snr_diff: TextDiff::new(snr),
            lli_diff:: TextDiff::new(lli),
        }
    } 
}