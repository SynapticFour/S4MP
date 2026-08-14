//! UTC timestamps for provenance and events.

use std::time::{SystemTime, UNIX_EPOCH};

/// Current UTC time as RFC-3339 with second precision (`YYYY-MM-DDTHH:MM:SSZ`).
#[must_use]
pub fn utc_rfc3339() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    unix_secs_to_rfc3339(secs)
}

/// Convert Unix epoch seconds to RFC-3339 UTC (`YYYY-MM-DDTHH:MM:SSZ`).
#[must_use]
pub fn unix_secs_to_rfc3339(secs: u64) -> String {
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let hour = rem / 3_600;
    let min = (rem % 3_600) / 60;
    let sec = rem % 60;
    let (year, month, day) = civil_from_unix_days(days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}

/// Civil date from days since Unix epoch. Howard Hinnant algorithm.
fn civil_from_unix_days(days: u64) -> (i32, u32, u32) {
    let z = i64::try_from(days).unwrap_or(i64::MAX) + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = u64::try_from(z - era * 146_097).unwrap_or(0);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = i64::try_from(yoe).unwrap_or(0) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (
        i32::try_from(y).unwrap_or(1970),
        u32::try_from(m).unwrap_or(1),
        u32::try_from(d).unwrap_or(1),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_is_rfc3339() {
        assert_eq!(unix_secs_to_rfc3339(0), "1970-01-01T00:00:00Z");
        assert_eq!(unix_secs_to_rfc3339(86_400), "1970-01-02T00:00:00Z");
        assert_eq!(unix_secs_to_rfc3339(1_000_000_000), "2001-09-09T01:46:40Z");
    }
}
