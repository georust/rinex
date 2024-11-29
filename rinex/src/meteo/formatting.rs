use std::{
    io::{BufWriter, Write},
    str::FromStr,
};

/// Formats Meteo epoch into [BufWriter]
pub fn format<W: Write>(
    w: &mut BufWriter<W>,
    record: &Record,
    header: &Header,
) -> Result<String, FormattingError> {
    let mut lines = String::with_capacity(128);
    lines.push_str(&format!(
        " {}",
        epoch::format(*epoch, Type::MeteoData, header.version.major)
    ));
    let observables = &header.meteo.as_ref().unwrap().codes;
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
