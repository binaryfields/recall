use sysinfo::System;

use crate::format;
use crate::plugins::{Output, Plugin};

pub struct Cpu {
    system: System,
    has_baseline: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            system: System::new(),
            has_baseline: false,
        }
    }
}

impl Plugin for Cpu {
    fn id(&self) -> &'static str {
        "cpu"
    }

    fn label(&self) -> &'static str {
        "CPU"
    }

    fn prefix(&self) -> &'static str {
        "C"
    }

    fn sample(&mut self) -> Output {
        self.system.refresh_cpu_usage();
        if !self.has_baseline {
            self.has_baseline = true;
            return Output::default();
        }
        let percent = format::percent_f64(self.system.global_cpu_usage() as f64);
        Output {
            segment: Some(percent.clone()),
            tooltip_lines: vec![format!("CPU: {percent}")],
            menu_items: None,
        }
    }

    fn invalidate(&mut self) {
        self.has_baseline = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_before_baseline() {
        let mut cpu = Cpu::new();
        assert!(cpu.sample().segment.is_none());
        assert!(cpu.sample().segment.is_some());
    }
}
