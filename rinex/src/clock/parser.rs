
/// Builds `RINEX` record entry for `Clocks` data files.   
/// Returns identified `epoch` to sort data efficiently.  
/// Returns 2D data as described in `record` definition
pub(crate) fn parse_epoch(
    version: Version,
    content: &str,
    ts: TimeScale,
) -> Result<(Epoch, ClockKey, ClockProfile), Error> {
    let mut lines = content.lines();
    let line = lines.next().unwrap();
    const LIMIT: Version = Version { major: 3, minor: 4 };
    let (dtype, mut rem) = line.split_at(3);
    let profile_type = ClockProfileType::from_str(dtype.trim())?;

    let clock_type = match version < LIMIT {
        true => {
            // old revision
            let (system_str, r) = rem.split_at(5);
            rem = r;
            if let Ok(svnn) = SV::from_str(system_str.trim()) {
                ClockType::SV(svnn)
            } else {
                ClockType::Station(system_str.trim().to_string())
            }
        },
        false => {
            // modern revision
            let (system_str, r) = rem.split_at(4);
            if let Ok(svnn) = SV::from_str(system_str.trim()) {
                let (_, r) = r.split_at(6);
                rem = r;
                ClockType::SV(svnn)
            } else {
                let mut content = system_str.to_owned();
                let (remainder, r) = r.split_at(6);
                rem = r;
                content.push_str(remainder);
                ClockType::Station(content.trim().to_string())
            }
        },
    };

    // Epoch: Y on 4 digits, even on RINEX2
    const OFFSET: usize = "yyyy mm dd hh mm sssssssssss".len();

    let (epoch, rem) = rem.split_at(OFFSET);
    let epoch = epoch::parse_in_timescale(epoch.trim(), ts)?;

    // nb of data fields
    let (_n, rem) = rem.split_at(4);

    // data fields
    let mut profile = ClockProfile::default();

    for (index, item) in rem.split_ascii_whitespace().enumerate() {
        match index {
            0 => {
                profile.bias = item
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| Error::ClockProfileParsing)?;
            },
            1 => {
                profile.bias_dev = Some(
                    item.trim()
                        .parse::<f64>()
                        .map_err(|_| Error::ClockProfileParsing)?,
                );
            },
            _ => {},
        }
    }
    for line in lines {
        for (index, item) in line.split_ascii_whitespace().enumerate() {
            match index {
                0 => {
                    profile.drift = Some(
                        item.trim()
                            .parse::<f64>()
                            .map_err(|_| Error::ClockProfileParsing)?,
                    );
                },
                1 => {
                    profile.drift_dev = Some(
                        item.trim()
                            .parse::<f64>()
                            .map_err(|_| Error::ClockProfileParsing)?,
                    );
                },
                2 => {
                    profile.drift_change = Some(
                        item.trim()
                            .parse::<f64>()
                            .map_err(|_| Error::ClockProfileParsing)?,
                    );
                },
                3 => {
                    profile.drift_change_dev = Some(
                        item.trim()
                            .parse::<f64>()
                            .map_err(|_| Error::ClockProfileParsing)?,
                    );
                },
                _ => {},
            }
        }
    }
    Ok((
        epoch,
        ClockKey {
            clock_type,
            profile_type,
        },
        profile,
    ))
}