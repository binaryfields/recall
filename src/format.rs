const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
const MIB: f64 = 1024.0 * 1024.0;

pub fn percent(used: u64, total: u64) -> String {
    if total == 0 {
        return "--%".to_string();
    }
    format!("{}%", (used * 100 + total / 2) / total)
}

pub fn gib(bytes: u64) -> String {
    format!("{:.1} G", bytes as f64 / GIB)
}

pub fn gib_compact(bytes: u64) -> String {
    format!("{:.1}G", bytes as f64 / GIB)
}

pub fn scaled(bytes: u64) -> String {
    let (gigs, mibs) = (bytes as f64 / GIB, bytes as f64 / MIB);
    if gigs >= 1.0 {
        format!("{gigs:.2} G")
    } else if mibs >= 1.0 {
        format!("{mibs:.0} M")
    } else {
        format!("{mibs:.1} M")
    }
}
