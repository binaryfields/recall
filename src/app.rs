use std::ops::ControlFlow;
use std::time::{Duration, Instant};

use tray_icon::menu::MenuId;

use crate::config::{Config, Readout};
use crate::format;
use crate::macos;
use crate::stats::Stats;
use crate::tray::{Action, Tray};

const PROCESS_EVERY: u64 = 5;

const TICK: Duration = Duration::from_secs(2);

const TOP_PROCESSES: usize = 8;

pub struct App {
    tray: Tray,
    stats: Stats,
    config: Config,
    next_tick: Instant,
    tick_count: u64,
}

impl App {
    pub fn new() -> Self {
        let config = Config::load();
        let mut app = Self {
            tray: Tray::new(config.readout, config.memory),
            stats: Stats::new(),
            config,
            next_tick: Instant::now(),
            tick_count: 0,
        };
        app.tick();
        app
    }

    pub fn next_tick(&self) -> Instant {
        self.next_tick
    }

    pub fn tick(&mut self) {
        self.refresh_display();
        if self.tick_count.is_multiple_of(PROCESS_EVERY) {
            self.refresh_processes();
        }
        self.tick_count += 1;
        self.next_tick = Instant::now() + TICK;
    }

    fn refresh_display(&mut self) {
        let mem = self.stats.refresh_memory();
        let pressure = macos::pressure_level();
        let title = match self.config.readout {
            Readout::Percent => format::percent(mem.used, mem.total),
            Readout::Absolute => format::gib_compact(mem.used),
        };
        self.tray.set_title(title);
        self.tray
            .set_tooltip(format::memory_tooltip(&mem, pressure));
    }

    fn refresh_processes(&mut self) {
        let mut procs = self.stats.refresh_processes(self.config.memory);
        procs.sort_unstable_by_key(|p| std::cmp::Reverse(p.memory));
        procs.truncate(TOP_PROCESSES);
        self.tray.set_processes(&procs);
    }

    pub fn handle_menu(&mut self, id: &MenuId) -> ControlFlow<()> {
        match self.tray.action(id) {
            Some(Action::ToggleFormat) => self.toggle_format(),
            Some(Action::ToggleMemory) => self.toggle_memory(),
            Some(Action::Quit) => return ControlFlow::Break(()),
            None => {}
        }
        ControlFlow::Continue(())
    }

    fn toggle_format(&mut self) {
        self.config.readout = self.config.readout.toggled();
        self.config.save();
        self.tray.set_readout(self.config.readout);
        self.refresh_display();
    }

    fn toggle_memory(&mut self) {
        self.config.memory = self.config.memory.toggled();
        self.config.save();
        self.tray.set_memory(self.config.memory);
        self.refresh_processes();
    }
}
