use sysinfo::System;

pub struct Stats {
    system: System,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            system: System::new(),
        }
    }

    pub fn refresh_memory(&mut self) -> Snapshot {
        self.system.refresh_memory();
        Snapshot {
            total: self.system.total_memory(),
            used: self.system.used_memory(),
            available: self.system.available_memory(),
            swap_total: self.system.total_swap(),
            swap_used: self.system.used_swap(),
        }
    }
}

pub struct Snapshot {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub swap_total: u64,
    pub swap_used: u64,
}
