//! Epoch formatting helper

/*
 * Epoch formatter
 * is used when we're dumping a Meteo RINEX record entry
 */
pub(crate) fn fmt_epoch(
    header: &Header,
    observations: &[RecordEntry],
) -> Result<String, Error> {
    
    let mut lines = String::with_capacity(128);
    lines.push_str(&format!(
        " {}",
        epoch::format(*epoch, Type::MeteoData, header.version.major)
    ));
    
    // retrieve system codes
    let observables = &header.meteo.as_ref()
        .ok_or(Error::MissingObservableSpecs)?
        .codes;
    
    let mut index = 0;
    for obscode in observables {
        index += 1;
        if let Some(data) = data.get(obscode) {
            lines.push_str(&format!("{:7.1}", data));
        } else {
            lines.push_str("       ");
        }
        if (index % 8) == 0 {
            lines.push('\n');
        }
    }
    
    lines.push('\n');
    Ok(lines)
}
