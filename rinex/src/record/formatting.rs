use crate::{
    doris::format as format_doris_observations,
    meteo::format as format_meteo_observations,
    navigation::format as format_navigation,
    observation::{
        format as format_observations, format_compressed as format_compressed_observations,
    },
    prelude::{FormattingError, Header},
    record::Record,
};

use std::io::{BufWriter, Write};

impl Record {
    pub fn format<W: Write>(
        &self,
        w: &mut BufWriter<W>,
        header: &Header,
    ) -> Result<(), FormattingError> {
        if let Some(rec) = self.as_obs() {
            if let Some(obs_header) = &header.obs {
                if obs_header.crinex.is_some() {
                    format_compressed_observations(w, rec, &obs_header)
                } else {
                    format_observations(w, rec, header)
                }
            } else {
                // missing observation specific fields
                // run through standard process
                // but this will rapidly panic
                format_observations(w, rec, header)
            }
        } else if let Some(rec) = self.as_meteo() {
            format_meteo_observations(w, rec, header)
        } else if let Some(rec) = self.as_doris() {
            format_doris_observations(w, rec, header)
        } else if let Some(rec) = self.as_nav() {
            format_navigation(w, rec, header)
        } else {
            Ok(())
        }
    }
}
