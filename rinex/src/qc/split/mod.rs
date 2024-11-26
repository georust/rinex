use crate::prelude::{qc::Split, Duration, Record, Rinex};

mod clock;
mod doris;
mod ionex;
mod meteo;
mod nav;
mod obs;

use obs::{split as obs_split, split_even_dt as obs_split_even_dt, split_mut as obs_split_mut};

use nav::{split as nav_split, split_even_dt as nav_split_even_dt, split_mut as nav_split_mut};

use meteo::{
    split as meteo_split, split_even_dt as meteo_split_even_dt, split_mut as meteo_split_mut,
};

use clock::{
    split as clock_split, split_even_dt as clock_split_even_dt, split_mut as clock_split_mut,
};

use ionex::{
    split as ionex_split, split_even_dt as ionex_split_even_dt, split_mut as ionex_split_mut,
};

use doris::{
    split as doris_split, split_even_dt as doris_split_even_dt, split_mut as doris_split_mut,
};

impl Split for Rinex {
    fn split(&self, t: hifitime::Epoch) -> (Self, Self) {
        let (r0, r1) = if let Some(r) = self.record.as_obs() {
            let (r0, r1) = obs_split(r, t);
            (Record::ObsRecord(r0), Record::ObsRecord(r1))
        } else if let Some(r) = self.record.as_nav() {
            let (r0, r1) = nav_split(r, t);
            (Record::NavRecord(r0), Record::NavRecord(r1))
        } else if let Some(r) = self.record.as_clock() {
            let (r0, r1) = clock_split(r, t);
            (Record::ClockRecord(r0), Record::ClockRecord(r1))
        } else if let Some(r) = self.record.as_ionex() {
            let (r0, r1) = ionex_split(r, t);
            (Record::IonexRecord(r0), Record::IonexRecord(r1))
        } else if let Some(r) = self.record.as_doris() {
            let (r0, r1) = doris_split(r, t);
            (Record::DorisRecord(r0), Record::DorisRecord(r1))
        } else if let Some(r) = self.record.as_meteo() {
            let (r0, r1) = meteo_split(r, t);
            (Record::MeteoRecord(r0), Record::MeteoRecord(r1))
        } else {
            panic!("non feasible split");
        };

        // TODO: improve this
        //  split comments timewise
        //  implement Split for Header
        //  implement Split for production attributes
        (
            Rinex {
                record: r0,
                header: self.header.clone(),
                comments: self.comments.clone(),
                prod_attr: self.prod_attr.clone(),
            },
            Rinex {
                record: r1,
                header: self.header.clone(),
                comments: self.comments.clone(),
                prod_attr: self.prod_attr.clone(),
            },
        )
    }

    fn split_mut(&mut self, t: hifitime::Epoch) -> Self {
        let record = if let Some(r) = self.record.as_mut_obs() {
            Record::ObsRecord(obs_split_mut(r, t))
        } else if let Some(r) = self.record.as_mut_nav() {
            Record::NavRecord(nav_split_mut(r, t))
        } else if let Some(r) = self.record.as_mut_clock() {
            Record::ClockRecord(clock_split_mut(r, t))
        } else if let Some(r) = self.record.as_mut_ionex() {
            Record::IonexRecord(ionex_split_mut(r, t))
        } else if let Some(r) = self.record.as_mut_doris() {
            Record::DorisRecord(doris_split_mut(r, t))
        } else if let Some(r) = self.record.as_mut_meteo() {
            Record::MeteoRecord(meteo_split_mut(r, t))
        } else {
            self.record.clone()
        };

        Self {
            record,
            header: self.header.clone(),
            comments: self.comments.clone(),
            prod_attr: self.prod_attr.clone(),
        }
    }

    fn split_even_dt(&self, dt: Duration) -> Vec<Self> {
        Default::default()
    }
}
