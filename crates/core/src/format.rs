//! Human-friendly formatting shared by every front-end.

/// Formats a byte count with binary units: `0 B`, `1.5 KiB`, `12.3 MiB`.
pub fn bytes(n: u64) -> String {
    const UNITS: [&str; 6] = ["KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
    if n < 1024 {
        return format!("{n} B");
    }
    let mut value = n as f64 / 1024.0;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if value >= 100.0 {
        format!("{value:.0} {}", UNITS[unit])
    } else if value >= 10.0 {
        format!("{value:.1} {}", UNITS[unit])
    } else {
        format!("{value:.2} {}", UNITS[unit])
    }
}

/// Formats a compression ratio (packed / size) as a percentage of the original.
pub fn ratio(size: u64, packed: u64) -> Option<String> {
    if size == 0 {
        return None;
    }
    let pct = packed as f64 / size as f64 * 100.0;
    Some(format!("{pct:.0}%"))
}

/// Formats a duration as `850 ms`, `12 s`, `3 min 05 s` or `1 h 02 min`.
pub fn duration(d: std::time::Duration) -> String {
    let secs = d.as_secs();
    if secs == 0 {
        return format!("{} ms", d.as_millis());
    }
    if secs < 60 {
        return format!("{secs} s");
    }
    if secs < 3600 {
        return format!("{} min {:02} s", secs / 60, secs % 60);
    }
    format!("{} h {:02} min", secs / 3600, (secs % 3600) / 60)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn byte_units() {
        assert_eq!(bytes(0), "0 B");
        assert_eq!(bytes(1023), "1023 B");
        assert_eq!(bytes(1536), "1.50 KiB");
        assert_eq!(bytes(15 * 1024 * 1024), "15.0 MiB");
        assert_eq!(bytes(300 * 1024 * 1024 * 1024), "300 GiB");
    }

    #[test]
    fn ratios() {
        assert_eq!(ratio(0, 10), None);
        assert_eq!(ratio(200, 50).as_deref(), Some("25%"));
    }

    #[test]
    fn durations() {
        assert_eq!(duration(Duration::from_millis(850)), "850 ms");
        assert_eq!(duration(Duration::from_secs(12)), "12 s");
        assert_eq!(duration(Duration::from_secs(185)), "3 min 05 s");
        assert_eq!(duration(Duration::from_mins(62)), "1 h 02 min");
    }
}
