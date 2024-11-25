use crate::{
    observation::format as format_observations,
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
            format_observations(w, rec, header)
        } else {
            Ok(())
        }
    }
}
