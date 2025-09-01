use crate::stats::Snapshot;

const GIB: f64 = 1024.0 * 1024.0 * 1024.0;

pub fn percent(used: u64, total: u64) -> String {
    if total == 0 {
        return "--%".to_string();
    }
    format!("{}%", (used * 100 + total / 2) / total)
}

pub fn memory_tooltip(mem: &Snapshot, pressure: &str) -> String {
    format!(
        "Total: {}\nUsed: {} ({})\nAvailable: {}\nSwap: {} / {}\nPressure: {}",
        gib(mem.total),
        gib(mem.used),
        percent(mem.used, mem.total),
        gib(mem.available),
        gib(mem.swap_used),
        gib(mem.swap_total),
        pressure,
    )
}

fn gib(bytes: u64) -> String {
    format!("{:.1} G", bytes as f64 / GIB)
}

pub fn gib_compact(bytes: u64) -> String {
    format!("{:.1}G", bytes as f64 / GIB)
}
