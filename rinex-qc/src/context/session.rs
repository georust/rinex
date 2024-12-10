use std::ops::Range;
use itertools::Itertools;
use crate::context::{QcContext, Error};

impl QcContext {

    /// Returns the Year range of this session, in chronological order
    fn year_range(&self) -> Option<Range<u16>> {
        Some(self.oldest_year()?..self.newest_year()?)
    }

    fn oldest_year(&self) -> Option<u16> {
        let min = self.all_meta_iter().map(|meta| meta.year).min();
        if min == 0 {
            None
        } else {
            Some(min)
        }
    }
    
    fn newest_year(&self) -> Option<u16> {
        let max = self.all_meta_iter().map(|meta| meta.year).max();
        if max == 0 {
            None
        } else {
            Some(max)
        }
    }
    
    fn oldest_doy(&self, year: u16) -> Option<u16> {
        let min = self.all_meta_iter().filter_map(|meta| meta.year == year).min();
        if min == 0 {
            None
        } else {
            Some(min)
        }
    }
    
    fn newest_doy(&self, year: u16) -> Option<u16> {
        let max = self.all_meta_iter().filter_map(|meta| meta.year == year).max();
        if max == 0 {
            None
        } else {
            Some(max)
        }
    }
    
    fn doy_range(&self, year: u16) -> Option<Range<u16>> {
        Some(self.oldest_doy(year)?..self.newest_doy(year)?)
    }

    /// Prepare and deploy [QcContext], at this point
    /// we're ready to generate data
    fn deploy(&self) -> Result<(), Error> {
        // make   
    }

    /// Creates the file hierarchy, within the workspace
    fn create_workspace(&self) -> Result<(), Error> {
        for year in self.year_range() {
            for doy in self.doy_range(year) {
                for product in self.all_products_iter() {
                    let fp = format!("{}/{}/{:x}", year, doy, product);
                    self.create_subdir()?;
                }
            }
        }
        trace!("workspace tree generated");
    }



}
