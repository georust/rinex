

/// Writes epoch into stream
pub(crate) fn fmt_epoch(epoch: &Epoch, key: &ClockKey, prof: &ClockProfile) -> String {
    let mut lines = String::with_capacity(60);
    let (y, m, d, hh, mm, ss, _) = epoch.to_gregorian_utc();

    let mut n = 1;
    if prof.drift.is_some() {
        n += 1;
    }
    if prof.drift_dev.is_some() {
        n += 1;
    }
    if prof.drift_change.is_some() {
        n += 1;
    }
    if prof.drift_change_dev.is_some() {
        n += 1;
    }

    lines.push_str(&format!(
        "{} {}  {} {:02} {:02} {:02} {:02} {:02}.000000  {}   {:.12E}",
        key.profile_type, key.clock_type, y, m, d, hh, mm, ss, n, prof.bias
    ));

    if let Some(sigma) = prof.bias_dev {
        lines.push_str(&format!("{:.13E} ", sigma));
    }
    lines.push('\n');
    if let Some(drift) = prof.drift {
        lines.push_str(&format!("   {:.13E} ", drift));
        if let Some(sigma) = prof.drift_dev {
            lines.push_str(&format!("{:.13E} ", sigma));
        }
        if let Some(drift_change) = prof.drift_change {
            lines.push_str(&format!("{:.13E} ", drift_change));
        }
        if let Some(sigma) = prof.drift_change_dev {
            lines.push_str(&format!("{:.13E} ", sigma));
        }
        lines.push('\n');
    }
    lines
}
