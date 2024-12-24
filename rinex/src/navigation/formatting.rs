/*
 * When formatting floating point number in Navigation RINEX,
 * exponent are expected to be in the %02d form,
 * but Rust is only capable of formating %d (AFAIK).
 * With this macro, we simply rework all exponents encountered in a string
 */
fn double_exponent_digits(content: &str) -> String {
    // replace "eN " with "E+0N"
    let re = Regex::new(r"e\d{1} ").unwrap();
    let lines = re.replace_all(content, |caps: &Captures| format!("E+0{}", &caps[0][1..]));

    // replace "eN" with "E+0N"
    let re = Regex::new(r"e\d{1}").unwrap();
    let lines = re.replace_all(&lines, |caps: &Captures| format!("E+0{}", &caps[0][1..]));

    // replace "e-N " with "E-0N"
    let re = Regex::new(r"e-\d{1} ").unwrap();
    let lines = re.replace_all(&lines, |caps: &Captures| format!("E-0{}", &caps[0][2..]));

    // replace "e-N" with "e-0N"
    let re = Regex::new(r"e-\d{1}").unwrap();
    let lines = re.replace_all(&lines, |caps: &Captures| format!("E-0{}", &caps[0][2..]));

    lines.to_string()
}


/*
 * Reworks generated/formatted line to match standards
 */
fn fmt_rework(major: u8, lines: &str) -> String {
    /*
     * There's an issue when formatting the exponent 00 in XXXXX.E00
     * Rust does not know how to format an exponent on multiples digits,
     * and RINEX expects two.
     * If we try to rework this line, it may corrupt some SVNN fields.
     */
    let mut lines = double_exponent_digits(lines);

    if major < 3 {
        /*
         * In old RINEX, D+00 D-01 is used instead of E+00 E-01
         */
        lines = lines.replace("E-", "D-");
        lines = lines.replace("E+", "D+");
    }
    lines.to_string()
}

/*
 * Writes given epoch into stream
 */
pub(crate) fn fmt_epoch(
    epoch: &Epoch,
    data: &Vec<NavFrame>,
    header: &Header,
) -> Result<String, FormattingError> {
    if header.version.major < 4 {
        fmt_epoch_v2v3(epoch, data, header)
    } else {
        fmt_epoch_v4(epoch, data, header)
    }
}

fn fmt_epoch_v2v3(
    epoch: &Epoch,
    data: &Vec<NavFrame>,
    header: &Header,
) -> Result<String, FormattingError> {
    let mut lines = String::with_capacity(128);
    for fr in data.iter() {
        if let Some(fr) = fr.as_eph() {
            let (_, sv, ephemeris) = fr;
            match &header.constellation {
                Some(Constellation::Mixed) => {
                    // Mixed constellation context
                    // we need to fully describe the vehicle
                    lines.push_str(&format!("{} ", sv));
                },
                Some(_) => {
                    // Unique constellation context:
                    // in V2 format, only PRN is shown
                    lines.push_str(&format!("{:2} ", sv.prn));
                },
                None => {
                    return Err(FormattingError::NoConstellationDefinition);
                },
            }
            lines.push_str(&format!(
                "{} ",
                epoch::format(*epoch, Type::NavigationData, header.version.major)
            ));
            lines.push_str(&format!(
                "{:14.11E} {:14.11E} {:14.11E}\n   ",
                ephemeris.clock_bias, ephemeris.clock_drift, ephemeris.clock_drift_rate
            ));
            if header.version.major == 3 {
                lines.push_str("  ");
            }

            // locate closest standards in DB
            let closest_orbits_definition =
                match closest_nav_standards(sv.constellation, header.version, NavMsgType::LNAV) {
                    Some(v) => v,
                    _ => return Err(FormattingError::NoNavigationDefinition),
                };

            let nb_items_per_line = 4;
            let mut chunks = closest_orbits_definition
                .items
                .chunks_exact(nb_items_per_line)
                .peekable();

            while let Some(chunk) = chunks.next() {
                if chunks.peek().is_some() {
                    for (key, _) in chunk {
                        if let Some(data) = ephemeris.orbits.get(*key) {
                            lines.push_str(&format!("{} ", data.to_string()));
                        } else {
                            lines.push_str("                   ");
                        }
                    }
                    lines.push_str("\n     ");
                } else {
                    // last row
                    for (key, _) in chunk {
                        if let Some(data) = ephemeris.orbits.get(*key) {
                            lines.push_str(&data.to_string());
                        } else {
                            lines.push_str("                   ");
                        }
                    }
                    lines.push('\n');
                }
            }
        }
    }
    lines = fmt_rework(header.version.major, &lines);
    Ok(lines)
}

fn fmt_epoch_v4(
    epoch: &Epoch,
    data: &Vec<NavFrame>,
    header: &Header,
) -> Result<String, FormattingError> {
    let mut lines = String::with_capacity(128);
    for fr in data.iter() {
        if let Some(fr) = fr.as_eph() {
            let (msgtype, sv, ephemeris) = fr;
            lines.push_str(&format!("> {} {} {}\n", FrameClass::Ephemeris, sv, msgtype));
            match &header.constellation {
                Some(Constellation::Mixed) => {
                    // Mixed constellation context
                    // we need to fully describe the vehicle
                    lines.push_str(&sv.to_string());
                    lines.push(' ');
                },
                Some(_) => {
                    // Unique constellation context:
                    // in V2 format, only PRN is shown
                    lines.push_str(&format!("{:02} ", sv.prn));
                },
                None => panic!("producing data with no constellation previously defined"),
            }
            lines.push_str(&format!(
                "{} ",
                epoch::format(*epoch, Type::NavigationData, header.version.major)
            ));
            lines.push_str(&format!(
                "{:14.13E} {:14.13E} {:14.13E}\n",
                ephemeris.clock_bias, ephemeris.clock_drift, ephemeris.clock_drift_rate
            ));

            // locate closest revision in DB
            let closest_orbits_definition =
                match closest_nav_standards(sv.constellation, header.version, NavMsgType::LNAV) {
                    Some(v) => v,
                    _ => return Err(FormattingError::NoNavigationDefinition),
                };

            let mut index = 0;
            for (key, _) in closest_orbits_definition.items.iter() {
                index += 1;
                if let Some(data) = ephemeris.orbits.get(*key) {
                    lines.push_str(&format!(" {}", data.to_string()));
                } else {
                    // data is missing: either not parsed or not provided
                    lines.push_str("              ");
                }
                if (index % 4) == 0 {
                    lines.push_str("\n   "); //TODO: do not TAB when writing last line of grouping
                }
            }
        } else if let Some(fr) = fr.as_sto() {
            let (msg, sv, sto) = fr;
            lines.push_str(&format!(
                "> {} {} {}\n",
                FrameClass::SystemTimeOffset,
                sv,
                msg
            ));
            lines.push_str(&format!(
                "    {} {}    {}\n",
                epoch::format(*epoch, Type::NavigationData, header.version.major),
                sto.system,
                sto.utc
            ));
            lines.push_str(&format!(
                "   {:14.13E} {:14.13E} {:14.13E} {:14.13E}\n",
                sto.t_tm as f64, sto.a.0, sto.a.1, sto.a.2
            ));
        } else if let Some(_fr) = fr.as_eop() {
            todo!("NAV V4: EOP: we have no example as of today");
            //(x, xr, xrr), (y, yr, yrr), t_tm, (dut, dutr, dutrr)) = frame.as_eop()
        }
        // EOP
        else if let Some(fr) = fr.as_ion() {
            let (msg, sv, ion) = fr;
            lines.push_str(&format!(
                "> {} {} {}\n",
                FrameClass::EarthOrientation,
                sv,
                msg
            ));
            match ion {
                IonMessage::KlobucharModel(_model) => todo!("ION:Kb"),
                IonMessage::NequickGModel(_model) => todo!("ION:Ng"),
                IonMessage::BdgimModel(_model) => todo!("ION:Bd"),
            }
        } // ION
    }
    lines = fmt_rework(4, &lines);
    Ok(lines)
}
