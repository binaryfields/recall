use std::time::{Duration, Instant};

use tray_icon::menu::MenuId;

use crate::format;
use crate::macos;
use crate::stats::Stats;
use crate::tray::{Action, Tray};

const TICK: Duration = Duration::from_secs(2);

pub struct App {
    tray: Tray,
    stats: Stats,
    next_tick: Instant,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            tray: Tray::new(),
            stats: Stats::new(),
            next_tick: Instant::now(),
        };
        app.tick();
        app
    }

    pub fn next_tick(&self) -> Instant {
        self.next_tick
    }

    pub fn tick(&mut self) {
        let mem = self.stats.refresh_memory();
        let pressure = macos::pressure_level();
        self.tray.set_title(format::percent(mem.used, mem.total));
        self.tray
            .set_tooltip(format::memory_tooltip(&mem, pressure));
        self.next_tick = Instant::now() + TICK;
    }

    pub fn dispatch(&mut self, id: &MenuId) -> bool {
        match self.tray.action(id) {
            Some(Action::Quit) => true,
            None => false,
        }
    }
}
