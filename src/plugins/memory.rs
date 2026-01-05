use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

use crate::format;
use crate::macos;
use crate::plugins::{Flag, Item, Output, Plugin};

const TOP_PROCESSES: usize = 8;

const PROCESS_EVERY: u64 = 5;

pub struct Memory {
    system: System,
    absolute: bool,
    rss: bool,
    samples: u64,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            system: System::new(),
            absolute: false,
            rss: false,
            samples: 0,
        }
    }

    fn process_items(&mut self, rss: bool) -> Vec<Item> {
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_memory(),
        );
        let mut procs: Vec<_> = self
            .system
            .processes()
            .values()
            .map(|process| {
                let pid = process.pid().as_u32() as libc::pid_t;
                let memory = if rss {
                    process.memory()
                } else {
                    macos::phys_footprint_for_pid(pid).unwrap_or_else(|| process.memory())
                };
                (pid, memory, process)
            })
            .collect();
        procs.sort_unstable_by_key(|&(_, memory, _)| std::cmp::Reverse(memory));
        procs.truncate(TOP_PROCESSES);
        procs
            .into_iter()
            .map(|(pid, memory, process)| {
                let name = macos::app_name_for_pid(pid)
                    .unwrap_or_else(|| process.name().to_string_lossy().into_owned());
                Item {
                    text: name,
                    badge: Some(format::scaled(memory)),
                }
            })
            .collect()
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
        let rss = self.rss;
        let menu_items = self
            .samples
            .is_multiple_of(PROCESS_EVERY)
            .then(|| self.process_items(rss));
        self.samples += 1;
        Output {
            segment: Some(segment),
            tooltip_lines,
            menu_items,
        }
    }

    fn invalidate(&mut self) {
        self.samples = 0;
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
            "rss" => {
                self.rss = on;
                self.samples = 0;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item_counts(memory: &mut Memory, ticks: usize) -> Vec<Option<usize>> {
        (0..ticks)
            .map(|_| memory.sample().menu_items.map(|items| items.len()))
            .collect()
    }

    #[test]
    fn sample_repeated_ticks() {
        let mut memory = Memory::new();
        let counts = item_counts(&mut memory, 7);
        assert!(counts[0].is_some());
        assert!(counts[1..5].iter().all(Option::is_none));
        assert!(counts[5].is_some());
        assert!(counts[6].is_none());
    }

    #[test]
    fn set_flag_absolute_and_rss() {
        let mut memory = Memory::new();
        item_counts(&mut memory, 3);
        memory.set_flag("absolute", true);
        assert!(memory.sample().menu_items.is_none());
        memory.set_flag("rss", true);
        assert!(memory.sample().menu_items.is_some());
    }
}
