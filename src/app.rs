use std::ops::ControlFlow;
use std::time::{Duration, Instant};

use tray_icon::menu::MenuId;

use crate::config::{Config, Readout};
use crate::format;
use crate::macos;
use crate::stats::Stats;
use crate::tray::{Action, Tray};

const TICK: Duration = Duration::from_secs(2);

pub struct App {
    tray: Tray,
    stats: Stats,
    config: Config,
    next_tick: Instant,
}

impl App {
    pub fn new() -> Self {
        let config = Config::load();
        let mut app = Self {
            tray: Tray::new(config.readout),
            stats: Stats::new(),
            config,
            next_tick: Instant::now(),
        };
        app.tick();
        app
    }

    pub fn next_tick(&self) -> Instant {
        self.next_tick
    }

    pub fn tick(&mut self) {
        self.refresh_display();
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

    pub fn handle_menu(&mut self, id: &MenuId) -> ControlFlow<()> {
        match self.tray.action(id) {
            Some(Action::ToggleFormat) => self.toggle_format(),
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
}
