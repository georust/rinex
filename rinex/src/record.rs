use std::collections::{BTreeMap, HashMap};
use std::io::prelude::*;
use thiserror::Error;

#[cfg(feature = "serde")]
use serde::Serialize;

use super::{
    antex, clocks,
    clocks::{ClockData, ClockDataType},
    hatanaka::{Compressor, Decompressor},
    header, ionex, is_rinex_comment, merge,
    merge::Merge,
    meteo, navigation, observation,
    reader::BufferedReader,
    split,
    split::Split,
    types::Type,
    writer::BufferedWriter,
    *,
};
use hifitime::Duration;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Record {
    /// ATX record, see [antex::record::Record]
    AntexRecord(antex::Record),
    /// Clock record, see [clocks::record::Record]
    ClockRecord(clocks::Record),
    /// IONEX (Ionosphere maps) record, see [ionex::record::Record]
    IonexRecord(ionex::Record),
    /// Meteo record, see [meteo::record::Record]
    MeteoRecord(meteo::Record),
    /// Navigation record, see [navigation::record::Record]
    NavRecord(navigation::Record),
    /// Observation record, see [observation::record::Record]
    ObsRecord(observation::Record),
}

/// Record comments are high level informations, sorted by epoch
/// (timestamp) of appearance. We deduce the "associated" timestamp from the
/// previosuly parsed epoch, when parsing the record.
pub type Comments = BTreeMap<Epoch, Vec<String>>;

impl Record {
    /// Unwraps self as ANTEX record
    pub fn as_antex(&self) -> Option<&antex::Record> {
        match self {
            Record::AntexRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as mutable reference to ANTEX record
    pub fn as_mut_antex(&mut self) -> Option<&mut antex::Record> {
        match self {
            Record::AntexRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as CLK record
    pub fn as_clock(&self) -> Option<&clocks::Record> {
        match self {
            Record::ClockRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as mutable CLK record
    pub fn as_mut_clock(&mut self) -> Option<&mut clocks::Record> {
        match self {
            Record::ClockRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as IONEX record
    pub fn as_ionex(&self) -> Option<&ionex::Record> {
        match self {
            Record::IonexRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as mutable IONEX record
    pub fn as_mut_ionex(&mut self) -> Option<&mut ionex::Record> {
        match self {
            Record::IonexRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as MET record
    pub fn as_meteo(&self) -> Option<&meteo::Record> {
        match self {
            Record::MeteoRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Returns mutable reference to Meteo record
    pub fn as_mut_meteo(&mut self) -> Option<&mut meteo::Record> {
        match self {
            Record::MeteoRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as NAV record
    pub fn as_nav(&self) -> Option<&navigation::Record> {
        match self {
            Record::NavRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Returns mutable reference to Navigation record
    pub fn as_mut_nav(&mut self) -> Option<&mut navigation::Record> {
        match self {
            Record::NavRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as OBS record
    pub fn as_obs(&self) -> Option<&observation::Record> {
        match self {
            Record::ObsRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Returns mutable reference to Observation record
    pub fn as_mut_obs(&mut self) -> Option<&mut observation::Record> {
        match self {
            Record::ObsRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Streams into given file writer
    pub fn to_file(
        &self,
        header: &header::Header,
        writer: &mut BufferedWriter,
    ) -> Result<(), Error> {
        match &header.rinex_type {
            Type::MeteoData => {
                let record = self.as_meteo().unwrap();
                for (epoch, data) in record.iter() {
                    if let Ok(epoch) = meteo::record::fmt_epoch(epoch, data, header) {
                        let _ = write!(writer, "{}", epoch);
                    }
                }
            },
            Type::ObservationData => {
                let record = self.as_obs().unwrap();
                let obs_fields = &header.obs.as_ref().unwrap();
                let mut compressor = Compressor::new();
                for ((epoch, flag), (clock_offset, data)) in record.iter() {
                    let epoch =
                        observation::record::fmt_epoch(*epoch, *flag, clock_offset, data, header);
                    if let Some(_) = &obs_fields.crinex {
                        let major = header.version.major;
                        let constell = &header.constellation.as_ref().unwrap();
                        for line in epoch.lines() {
                            let line = line.to_owned() + "\n"; // helps the following .lines() iterator
                                                               // embedded in compression method
                            if let Ok(compressed) =
                                compressor.compress(major, &obs_fields.codes, constell, &line)
                            {
                                write!(writer, "{}", compressed)?;
                            }
                        }
                    } else {
                        write!(writer, "{}", epoch)?;
                    }
                }
            },
            Type::NavigationData => {
                let record = self.as_nav().unwrap();
                for (epoch, frames) in record.iter() {
                    if let Ok(epoch) = navigation::record::fmt_epoch(epoch, frames, header) {
                        let _ = write!(writer, "{}", epoch);
                    }
                }
            },
            Type::ClockData => {
                if let Some(r) = self.as_clock() {
                    for (epoch, data) in r {
                        if let Ok(epoch) = clocks::record::fmt_epoch(epoch, data) {
                            let _ = write!(writer, "{}", epoch);
                        }
                    }
                }
            },
            Type::IonosphereMaps => {
                if let Some(r) = self.as_ionex() {
                    //for (index, (epoch, (_map, _, _))) in r.iter().enumerate() {
                    //    let _ = write!(writer, "{:6}                                                      START OF TEC MAP", index);
                    //    let _ = write!(
                    //        writer,
                    //        "{}                        EPOCH OF CURRENT MAP",
                    //        epoch::format(*epoch, None, Type::IonosphereMaps, 1)
                    //    );
                    //    let _ = write!(writer, "{:6}                                                      END OF TEC MAP", index);
                    //}
                    // /*
                    //  * not efficient browsing, but matches provided examples and common formatting.
                    //  * RMS and Height maps are passed after TEC maps.
                    //  */
                    //for (index, (epoch, (_, _map, _))) in r.iter().enumerate() {
                    //    let _ = write!(writer, "{:6}                                                      START OF RMS MAP", index);
                    //    let _ = write!(
                    //        writer,
                    //        "{}                        EPOCH OF CURRENT MAP",
                    //        epoch::format(*epoch, None, Type::IonosphereMaps, 1)
                    //    );
                    //    let _ = write!(writer, "{:6}                                                      END OF RMS MAP", index);
                    //}
                    //for (index, (epoch, (_, _, _map))) in r.iter().enumerate() {
                    //    let _ = write!(writer, "{:6}                                                      START OF HEIGHT MAP", index);
                    //    let _ = write!(
                    //        writer,
                    //        "{}                        EPOCH OF CURRENT MAP",
                    //        epoch::format(*epoch, None, Type::IonosphereMaps, 1)
                    //    );
                    //    let _ = write!(writer, "{:6}                                                      END OF HEIGHT MAP", index);
                    //}
                }
            },
            _ => panic!("record type not supported yet"),
        }
        Ok(())
    }
}

impl Default for Record {
    fn default() -> Record {
        Record::NavRecord(navigation::Record::new())
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("record parsing not supported for type \"{0}\"")]
    TypeError(String),
    #[error("file i/o error")]
    FileIoError(#[from] std::io::Error),
    #[error("failed to produce Navigation epoch")]
    NavEpochError(#[from] navigation::Error),
    #[error("failed to produce Clock epoch")]
    ClockEpochError(#[from] clocks::Error),
    #[error("missing TIME OF FIRST OBS")]
    BadObservationDataDefinition,
    #[error("failed to identify timescale")]
    ObservationDataTimescaleIdentification,
}

/// Returns true if given line matches the start   
/// of a new epoch, inside a RINEX record.
pub fn is_new_epoch(line: &str, header: &header::Header) -> bool {
    if is_rinex_comment(line) {
        return false;
    }
    match &header.rinex_type {
        Type::AntennaData => antex::record::is_new_epoch(line),
        Type::ClockData => clocks::record::is_new_epoch(line),
        Type::IonosphereMaps => {
            ionex::record::is_new_tec_plane(line) || ionex::record::is_new_rms_plane(line)
        },
        Type::NavigationData => navigation::record::is_new_epoch(line, header.version),
        Type::ObservationData => observation::record::is_new_epoch(line, header.version),
        Type::MeteoData => meteo::record::is_new_epoch(line, header.version),
    }
}

/// Builds a `Record`, `RINEX` file body content,
/// which is constellation and `RINEX` file type dependent
pub fn parse_record(
    reader: &mut BufferedReader,
    header: &mut header::Header,
) -> Result<(Record, Comments), Error> {
    let mut first_epoch = true;
    let mut content = String::default();
    let mut epoch_content = String::with_capacity(6 * 64);

    // to manage `record` comments
    let mut comments: Comments = Comments::new();
    let mut comment_ts = Epoch::default();
    let mut comment_content: Vec<String> = Vec::with_capacity(4);

    let mut decompressor = Decompressor::new();
    // record
    let mut atx_rec = antex::Record::new(); // ATX
    let mut nav_rec = navigation::Record::new(); // NAV
    let mut obs_rec = observation::Record::new(); // OBS
    let mut met_rec = meteo::Record::new(); // MET
    let mut clk_rec = clocks::Record::new(); // CLK

    // OBSERVATION case
    //  timescale is defined either
    //    [+] by TIME OF FIRST header field
    //    [+] fixed system in case of old GPS/GLO Observation Data
    let mut obs_ts = TimeScale::default();
    if let Some(obs) = &header.obs {
        match header.constellation {
            Some(Constellation::Mixed) | None => {
                let time_of_first_obs = obs
                    .time_of_first_obs
                    .ok_or(Error::BadObservationDataDefinition)?;
                obs_ts = time_of_first_obs.time_scale;
            },
            Some(constellation) => {
                obs_ts = constellation
                    .timescale()
                    .ok_or(Error::ObservationDataTimescaleIdentification)?;
            },
        }
    }
    // IONEX case
    //  Default map type is TEC, it will come with identified Epoch
    //  but others may exist:
    //    in this case we used the previously identified Epoch
    //    and attach other kinds of maps
    let mut ionx_rec = ionex::Record::new();
    let mut ionex_rms_plane = false;

    for l in reader.lines() {
        // iterates one line at a time
        let line = l.unwrap();
        // COMMENTS special case
        // --> store
        // ---> append later with epoch.timestamp attached to it
        if is_rinex_comment(&line) {
            let comment = line.split_at(60).0.trim_end();
            comment_content.push(comment.to_string());
            continue;
        }
        // IONEX exponent-->data scaling use update regularly
        //  and used in TEC map parsing
        if line.contains("EXPONENT") {
            if let Some(ionex) = header.ionex.as_mut() {
                let content = line.split_at(60).0;
                if let Ok(e) = i8::from_str_radix(content.trim(), 10) {
                    *ionex = ionex.with_exponent(e); // scaling update
                }
            }
        }
        /*
         * If plain RINEX: content is passed as is
         *      if CRINEX: decompress and pass recovered content
         */

        if let Some(obs) = &header.obs {
            if let Some(crinex) = &obs.crinex {
                /*
                 * CRINEX
                 */
                let constellation = &header.constellation.as_ref().unwrap();
                if let Ok(recovered) = decompressor.decompress(
                    crinex.version.major,
                    constellation,
                    header.version.major,
                    &obs.codes,
                    // we might encounter empty lines
                    //   like missing clock offsets
                    //   and .lines() will destroy them
                    &(line.to_owned() + "\n"),
                ) {
                    content = recovered.clone();
                } else {
                    content.clear();
                }
            } else {
                /*
                 * RINEX
                 */
                if line.len() == 0 {
                    // we might encounter empty lines
                    // and the following parsers (.lines() iterator)
                    // do not like it
                    content = String::from("\n");
                } else {
                    content = line.to_string();
                }
            }
        } else {
            /*
             * RINEX
             */
            if line.len() == 0 {
                // we might encounter empty lines
                // and the following parsers (.lines() iterator)
                // do not like it
                content = String::from("\n");
            } else {
                content = line.to_string();
            }
        }

        for line in content.lines() {
            // in case of CRINEX -> RINEX < 3 being recovered,
            // we have more than 1 ligne to process
            let new_epoch = is_new_epoch(line, &header);
            ionex_rms_plane = ionex::record::is_new_rms_plane(line);

            if new_epoch && !first_epoch {
                match &header.rinex_type {
                    Type::NavigationData => {
                        let constellation = &header.constellation.unwrap();
                        if let Ok((e, fr)) = navigation::record::parse_epoch(
                            header.version,
                            *constellation,
                            &epoch_content,
                        ) {
                            nav_rec
                                .entry(e)
                                .and_modify(|frames| frames.push(fr.clone()))
                                .or_insert_with(|| vec![fr.clone()]);
                            comment_ts = e.clone(); // for comments classification & management
                        }
                    },
                    Type::ObservationData => {
                        if let Ok((e, ck_offset, map)) =
                            observation::record::parse_epoch(&header, &epoch_content, obs_ts)
                        {
                            obs_rec.insert(e, (ck_offset, map));
                            comment_ts = e.0.clone(); // for comments classification & management
                        }
                    },
                    Type::MeteoData => {
                        if let Ok((e, map)) = meteo::record::parse_epoch(&header, &epoch_content) {
                            met_rec.insert(e, map);
                            comment_ts = e.clone(); // for comments classification & management
                        }
                    },
                    Type::ClockData => {
                        if let Ok((epoch, dtype, system, data)) =
                            clocks::record::parse_epoch(header.version, &epoch_content)
                        {
                            if let Some(e) = clk_rec.get_mut(&epoch) {
                                if let Some(d) = e.get_mut(&dtype) {
                                    d.insert(system, data);
                                } else {
                                    // --> new system entry for this `epoch`
                                    let mut inner: HashMap<clocks::System, ClockData> =
                                        HashMap::new();
                                    inner.insert(system, data);
                                    e.insert(dtype, inner);
                                }
                            } else {
                                // --> new epoch entry
                                let mut inner: HashMap<clocks::System, ClockData> = HashMap::new();
                                inner.insert(system, data);
                                let mut map: HashMap<
                                    ClockDataType,
                                    HashMap<clocks::System, ClockData>,
                                > = HashMap::new();
                                map.insert(dtype, inner);
                                clk_rec.insert(epoch, map);
                            }
                            comment_ts = epoch.clone(); // for comments classification & management
                        }
                    },
                    Type::AntennaData => {
                        if let Ok((antenna, frequencies)) =
                            antex::record::parse_epoch(&epoch_content)
                        {
                            let mut found = false;
                            for (ant, freqz) in atx_rec.iter_mut() {
                                if *ant == antenna {
                                    for f in frequencies.iter() {
                                        freqz.push(f.clone());
                                    }
                                    found = true;
                                    break;
                                }
                            }
                            if !found {
                                atx_rec.push((antenna, frequencies));
                            }
                        }
                    },
                    Type::IonosphereMaps => {
                        if let Ok((epoch, altitude, plane)) =
                            ionex::record::parse_plane(&epoch_content, header, ionex_rms_plane)
                        {
                            if ionex_rms_plane {
                                if let Some(rec_plane) = ionx_rec.get_mut(&(epoch, altitude)) {
                                    // provide RMS value for the entire plane
                                    for ((_, rec_tec), (_, tec)) in
                                        rec_plane.iter_mut().zip(plane.iter())
                                    {
                                        rec_tec.rms = tec.rms;
                                    }
                                } else {
                                    // insert RMS values
                                    ionx_rec.insert((epoch, altitude), plane);
                                }
                            } else if let Some(rec_plane) = ionx_rec.get_mut(&(epoch, altitude)) {
                                // provide TEC value for the entire plane
                                for ((_, rec_tec), (_, tec)) in
                                    rec_plane.iter_mut().zip(plane.iter())
                                {
                                    rec_tec.tec = tec.tec;
                                }
                            } else {
                                // insert TEC values
                                ionx_rec.insert((epoch, altitude), plane);
                            }
                        }
                    },
                }

                // new comments ?
                if !comment_content.is_empty() {
                    comments.insert(comment_ts, comment_content.clone());
                    comment_content.clear() // reset
                }
            } //is_new_epoch() +!first

            if new_epoch {
                if !first_epoch {
                    epoch_content.clear()
                }
                first_epoch = false;
            }
            // epoch content builder
            epoch_content.push_str(&(line.to_owned() + "\n"));
        }
    }

    // --> try to build an epoch out of current residues
    // this covers
    //   + final epoch (last epoch in record)
    //   + comments parsing with empty record (empty file body)
    match &header.rinex_type {
        Type::NavigationData => {
            let constellation = &header.constellation.unwrap();
            if let Ok((e, fr)) =
                navigation::record::parse_epoch(header.version, *constellation, &epoch_content)
            {
                nav_rec
                    .entry(e)
                    .and_modify(|current| current.push(fr.clone()))
                    .or_insert_with(|| vec![fr.clone()]);
                comment_ts = e.clone(); // for comments classification & management
            }
        },
        Type::ObservationData => {
            if let Ok((e, ck_offset, map)) =
                observation::record::parse_epoch(&header, &epoch_content, obs_ts)
            {
                obs_rec.insert(e, (ck_offset, map));
                comment_ts = e.0.clone(); // for comments classification + management
            }
        },
        Type::MeteoData => {
            if let Ok((e, map)) = meteo::record::parse_epoch(&header, &epoch_content) {
                met_rec.insert(e, map);
                comment_ts = e.clone(); // for comments classification + management
            }
        },
        Type::ClockData => {
            if let Ok((e, dtype, system, data)) =
                clocks::record::parse_epoch(header.version, &epoch_content)
            {
                // Clocks `RINEX` files are handled a little different,
                // because we parse one line at a time, while we parsed one epoch at a time for other RINEXes.
                // One line may contribute to a previously existing epoch in the record
                // (different type of measurements etc..etc..)
                if let Some(e) = clk_rec.get_mut(&e) {
                    if let Some(d) = e.get_mut(&dtype) {
                        d.insert(system, data);
                    } else {
                        // --> new system entry for this `epoch`
                        let mut map: HashMap<
                            ClockDataType,
                            HashMap<clocks::System, clocks::ClockData>,
                        > = HashMap::new();
                        let mut inner: HashMap<clocks::System, ClockData> = HashMap::new();
                        inner.insert(system, data);
                        map.insert(dtype, inner);
                    }
                } else {
                    // --> new epoch entry
                    let mut map: HashMap<ClockDataType, HashMap<clocks::System, ClockData>> =
                        HashMap::new();
                    let mut inner: HashMap<clocks::System, ClockData> = HashMap::new();
                    inner.insert(system, data);
                    map.insert(dtype, inner);
                    clk_rec.insert(e, map);
                }
                comment_ts = e.clone(); // for comments classification & management
            }
        },
        Type::IonosphereMaps => {
            if let Ok((epoch, altitude, plane)) =
                ionex::record::parse_plane(&epoch_content, header, ionex_rms_plane)
            {
                if ionex_rms_plane {
                    if let Some(rec_plane) = ionx_rec.get_mut(&(epoch, altitude)) {
                        // provide RMS value for the entire plane
                        for ((_, rec_tec), (_, tec)) in rec_plane.iter_mut().zip(plane.iter()) {
                            rec_tec.rms = tec.rms;
                        }
                    } else {
                        // insert RMS values
                        ionx_rec.insert((epoch, altitude), plane);
                    }
                } else if let Some(rec_plane) = ionx_rec.get_mut(&(epoch, altitude)) {
                    // provide TEC value for the entire plane
                    for ((_, rec_tec), (_, tec)) in rec_plane.iter_mut().zip(plane.iter()) {
                        rec_tec.tec = tec.tec;
                    }
                } else {
                    // insert TEC values
                    ionx_rec.insert((epoch, altitude), plane);
                }
            }
        },
        Type::AntennaData => {
            if let Ok((antenna, frequencies)) = antex::record::parse_epoch(&epoch_content) {
                let mut found = false;
                for (ant, freqz) in atx_rec.iter_mut() {
                    if *ant == antenna {
                        for f in frequencies.iter() {
                            freqz.push(f.clone());
                        }
                        found = true;
                        break;
                    }
                }
                if !found {
                    atx_rec.push((antenna, frequencies));
                }
            }
        },
    }
    // new comments ?
    if !comment_content.is_empty() {
        comments.insert(comment_ts, comment_content.clone());
    }
    // wrap record
    let record = match &header.rinex_type {
        Type::AntennaData => Record::AntexRecord(atx_rec),
        Type::ClockData => Record::ClockRecord(clk_rec),
        Type::IonosphereMaps => Record::IonexRecord(ionx_rec),
        Type::MeteoData => Record::MeteoRecord(met_rec),
        Type::NavigationData => Record::NavRecord(nav_rec),
        Type::ObservationData => Record::ObsRecord(obs_rec),
    };
    Ok((record, comments))
}

impl Merge for Record {
    /// Merges `rhs` into `Self` without mutable access at the expense of more memcopies
    fn merge(&self, rhs: &Self) -> Result<Self, merge::Error> {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
    /// Merges `rhs` into `Self`
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), merge::Error> {
        if let Some(lhs) = self.as_mut_nav() {
            if let Some(rhs) = rhs.as_nav() {
                lhs.merge_mut(&rhs)?;
            }
        } else if let Some(lhs) = self.as_mut_obs() {
            if let Some(rhs) = rhs.as_obs() {
                lhs.merge_mut(&rhs)?;
            }
        } else if let Some(lhs) = self.as_mut_meteo() {
            if let Some(rhs) = rhs.as_meteo() {
                lhs.merge_mut(&rhs)?;
            }
        /*} else if let Some(lhs) = self.as_mut_ionex() {
        if let Some(rhs) = rhs.as_ionex() {
            lhs.merge_mut(&rhs)?;
        }*/
        } else if let Some(lhs) = self.as_mut_antex() {
            if let Some(rhs) = rhs.as_antex() {
                lhs.merge_mut(&rhs)?;
            }
        } else if let Some(lhs) = self.as_mut_clock() {
            if let Some(rhs) = rhs.as_clock() {
                lhs.merge_mut(&rhs)?;
            }
        }
        Ok(())
    }
}

impl Split for Record {
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), split::Error> {
        if let Some(r) = self.as_obs() {
            let (r0, r1) = r.split(epoch)?;
            Ok((Self::ObsRecord(r0), Self::ObsRecord(r1)))
        } else if let Some(r) = self.as_nav() {
            let (r0, r1) = r.split(epoch)?;
            Ok((Self::NavRecord(r0), Self::NavRecord(r1)))
        } else if let Some(r) = self.as_meteo() {
            let (r0, r1) = r.split(epoch)?;
            Ok((Self::MeteoRecord(r0), Self::MeteoRecord(r1)))
        } else if let Some(r) = self.as_ionex() {
            let (r0, r1) = r.split(epoch)?;
            Ok((Self::IonexRecord(r0), Self::IonexRecord(r1)))
        } else if let Some(r) = self.as_clock() {
            let (r0, r1) = r.split(epoch)?;
            Ok((Self::ClockRecord(r0), Self::ClockRecord(r1)))
        } else {
            Err(split::Error::NoEpochIteration)
        }
    }
    fn split_dt(&self, _dt: Duration) -> Result<Vec<Self>, split::Error> {
        Ok(Vec::new())
    }
}

#[cfg(feature = "processing")]
use crate::algorithm::{Filter, Preprocessing};

#[cfg(feature = "processing")]
impl Preprocessing for Record {
    fn filter(&self, f: Filter) -> Self {
        let mut s = self.clone();
        s.filter_mut(f);
        s
    }
    fn filter_mut(&mut self, f: Filter) {
        if let Some(r) = self.as_mut_obs() {
            r.filter_mut(f);
        } else if let Some(r) = self.as_mut_nav() {
            r.filter_mut(f);
        } else if let Some(r) = self.as_mut_clock() {
            r.filter_mut(f);
        } else if let Some(r) = self.as_mut_meteo() {
            r.filter_mut(f);
        } else if let Some(r) = self.as_mut_ionex() {
            r.filter_mut(f);
        }
    }
}

#[cfg(feature = "processing")]
use crate::algorithm::Decimate;

#[cfg(feature = "processing")]
impl Decimate for Record {
    fn decimate_by_ratio(&self, r: u32) -> Self {
        let mut s = self.clone();
        s.decimate_by_ratio_mut(r);
        s
    }
    fn decimate_by_ratio_mut(&mut self, r: u32) {
        if let Some(rec) = self.as_mut_obs() {
            rec.decimate_by_ratio_mut(r);
        } else if let Some(rec) = self.as_mut_nav() {
            rec.decimate_by_ratio_mut(r);
        } else if let Some(rec) = self.as_mut_meteo() {
            rec.decimate_by_ratio_mut(r);
        }
    }
    fn decimate_by_interval(&self, dt: Duration) -> Self {
        let mut s = self.clone();
        s.decimate_by_interval_mut(dt);
        s
    }
    fn decimate_by_interval_mut(&mut self, dt: Duration) {
        if let Some(rec) = self.as_mut_obs() {
            rec.decimate_by_interval_mut(dt);
        } else if let Some(rec) = self.as_mut_nav() {
            rec.decimate_by_interval_mut(dt);
        } else if let Some(rec) = self.as_mut_meteo() {
            rec.decimate_by_interval_mut(dt);
        }
    }
    fn decimate_match(&self, rhs: &Self) -> Self {
        let mut s = self.clone();
        s.decimate_match_mut(rhs);
        s
    }
    fn decimate_match_mut(&mut self, rhs: &Self) {
        if let Some(rec) = self.as_mut_obs() {
            if let Some(rhs) = rhs.as_obs() {
                rec.decimate_match_mut(rhs);
            }
        } else if let Some(rec) = self.as_mut_nav() {
            if let Some(rhs) = rhs.as_nav() {
                rec.decimate_match_mut(rhs);
            }
        } else if let Some(rec) = self.as_mut_meteo() {
            if let Some(rhs) = rhs.as_meteo() {
                rec.decimate_match_mut(rhs);
            }
        }
    }
}
