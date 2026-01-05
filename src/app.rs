use std::ops::ControlFlow;
use std::time::{Duration, Instant};

use tray_icon::menu::MenuId;

use crate::config::Config;
use crate::macos;
use crate::plugins::{self, Output, Plugin};
use crate::tray::{Action, Tray};

const TICK: Duration = Duration::from_secs(2);

pub struct App {
    tray: Tray,
    config: Config,
    plugins: Vec<PluginSlot>,
    next_tick: Instant,
}

impl App {
    pub fn new() -> Self {
        let config = Config::load();
        let mut loaded = plugins::all();
        for plugin in &mut loaded {
            for flag in plugin.flags() {
                let on = config.flag(plugin.id(), flag.id).unwrap_or(flag.on);
                plugin.set_flag(flag.id, on);
            }
        }
        let tray = Tray::new(&loaded);
        let plugins = loaded
            .into_iter()
            .map(|plugin| PluginSlot {
                plugin,
                output: Output::default(),
            })
            .collect();
        let mut app = Self {
            tray,
            config,
            plugins,
            next_tick: Instant::now(),
        };
        app.tick();
        app
    }

    pub fn next_tick(&self) -> Instant {
        self.next_tick
    }

    pub fn tick(&mut self) {
        self.sample_all();
        self.next_tick = Instant::now() + TICK;
    }

    fn sample_all(&mut self) {
        for plugin in 0..self.plugins.len() {
            self.sample_slot(plugin);
        }
        self.render_status();
    }

    fn sample_slot(&mut self, plugin: usize) {
        let slot = &mut self.plugins[plugin];
        let output = slot.plugin.sample();
        slot.output.segment = output.segment;
        slot.output.tooltip_lines = output.tooltip_lines;
        if let Some(items) = output.menu_items {
            self.tray.set_section(plugin, items);
        }
    }

    fn render_status(&self) {
        let mut segments = Vec::new();
        let mut blocks = Vec::new();
        for slot in &self.plugins {
            if let Some(segment) = &slot.output.segment {
                segments.push((slot.plugin.prefix(), segment.as_str()));
            }
            if !slot.output.tooltip_lines.is_empty() {
                blocks.push(slot.output.tooltip_lines.join("\n"));
            }
        }
        self.tray.set_title(status_title(&segments));
        self.tray.set_tooltip(blocks.join("\n\n"));
    }

    pub fn handle_menu(&mut self, id: &MenuId) -> ControlFlow<()> {
        match self.tray.action(id) {
            Some(Action::ToggleFlag(plugin, flag)) => self.toggle_flag(plugin, flag),
            Some(Action::ToggleLogin) => self.toggle_login(),
            Some(Action::Quit) => return ControlFlow::Break(()),
            None => {}
        }
        ControlFlow::Continue(())
    }

    fn toggle_flag(&mut self, plugin: usize, flag: &str) {
        let slot = &mut self.plugins[plugin];
        let on = !slot
            .plugin
            .flags()
            .iter()
            .any(|known| known.id == flag && known.on);
        slot.plugin.set_flag(flag, on);
        self.config.set_flag(slot.plugin.id(), flag, on);
        self.config.save();
        self.sample_slot(plugin);
        self.render_status();
    }

    fn toggle_login(&mut self) {
        let enabled = macos::set_login_enabled(!macos::login_enabled());
        self.tray.set_login_checked(enabled);
    }
}

struct PluginSlot {
    plugin: Box<dyn Plugin>,
    output: Output,
}

fn status_title(segments: &[(&str, &str)]) -> String {
    if segments.is_empty() {
        "\u{2026}".to_string()
    } else {
        segments
            .iter()
            .map(|(prefix, segment)| format!("{prefix}{segment}"))
            .collect::<Vec<_>>()
            .join(" ")
    }
}
