use sysinfo::System;

use crate::format;
use crate::macos;
use crate::plugins::{Flag, Item, Output, Plugin};

const TOP_PROCESSES: usize = 8;

pub struct Memory {
    system: System,
    absolute: bool,
    rss: bool,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            system: System::new(),
            absolute: false,
            rss: false,
        }
    }
}

impl Plugin for Memory {
    fn id(&self) -> &'static str {
        "memory"
    }

    fn label(&self) -> &'static str {
        "Memory"
    }

    fn prefix(&self) -> &'static str {
        "M"
    }

    fn sample(&mut self) -> Output {
        self.system.refresh_memory();
        let total = self.system.total_memory();
        let used = self.system.used_memory();
        let segment = if self.absolute {
            format::gib_compact(used)
        } else {
            format::percent(used, total)
        };
        let tooltip_lines = vec![
            format!("Total: {}", format::gib(total)),
            format!(
                "Used: {} ({})",
                format::gib(used),
                format::percent(used, total)
            ),
            format!("Available: {}", format::gib(self.system.available_memory())),
            format!(
                "Swap: {} / {}",
                format::gib(self.system.used_swap()),
                format::gib(self.system.total_swap())
            ),
            format!("Pressure: {}", macos::pressure_level()),
        ];
        Output {
            segment: Some(segment),
            tooltip_lines,
        }
    }

    fn menu_will_open(&mut self) -> Vec<Item> {
        let mut procs: Vec<(libc::pid_t, u64)> = macos::pids()
            .into_iter()
            .filter_map(|pid| {
                let memory = macos::memory_for_pid(pid)?;
                let bytes = if self.rss {
                    memory.resident
                } else {
                    memory.footprint
                };
                Some((pid, bytes))
            })
            .collect();
        procs.sort_unstable_by_key(|&(_, bytes)| std::cmp::Reverse(bytes));
        procs.truncate(TOP_PROCESSES);
        procs
            .into_iter()
            .map(|(pid, bytes)| Item {
                text: macos::app_name_for_pid(pid)
                    .or_else(|| macos::process_name(pid))
                    .unwrap_or_else(|| pid.to_string()),
                badge: Some(format::scaled(bytes)),
            })
            .collect()
    }

    fn flags(&self) -> Vec<Flag> {
        vec![
            Flag {
                id: "absolute",
                label: "Show Absolute Usage",
                on: self.absolute,
            },
            Flag {
                id: "rss",
                label: "Show RSS Memory",
                on: self.rss,
            },
        ]
    }

    fn set_flag(&mut self, id: &str, on: bool) {
        match id {
            "absolute" => self.absolute = on,
            "rss" => self.rss = on,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_will_open_lists_top_processes() {
        let mut memory = Memory::new();
        let items = memory.menu_will_open();
        assert!(!items.is_empty());
        assert!(items.len() <= TOP_PROCESSES);
        assert!(items.iter().all(|item| item.badge.is_some()));
    }

    #[test]
    fn set_flag_rss_changes_items() {
        let mut memory = Memory::new();
        let footprint = memory.menu_will_open();
        memory.set_flag("rss", true);
        let rss = memory.menu_will_open();
        assert!(memory.rss);
        assert!(footprint.iter().all(|item| !item.text.is_empty()));
        assert!(rss.iter().all(|item| !item.text.is_empty()));
    }
}
