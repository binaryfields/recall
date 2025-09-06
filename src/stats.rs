use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

use crate::config::MemoryMetric;
use crate::macos;

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

    pub fn refresh_processes(&mut self, metric: MemoryMetric) -> Vec<ProcInfo> {
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_memory(),
        );
        self.system
            .processes()
            .values()
            .map(|process| {
                let pid = process.pid().as_u32() as libc::pid_t;
                let name = macos::app_name_for_pid(pid)
                    .unwrap_or_else(|| process.name().to_string_lossy().into_owned());
                let memory = match metric {
                    MemoryMetric::Footprint => {
                        macos::phys_footprint_for_pid(pid).unwrap_or_else(|| process.memory())
                    }
                    MemoryMetric::Rss => process.memory(),
                };
                ProcInfo { name, memory }
            })
            .collect()
    }
}

pub struct Snapshot {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub swap_total: u64,
    pub swap_used: u64,
}

pub struct ProcInfo {
    pub name: String,
    pub memory: u64,
}
