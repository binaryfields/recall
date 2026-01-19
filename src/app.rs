use std::ops::ControlFlow;
use std::time::{Duration, Instant};

use tray_icon::menu::MenuId;

use crate::config::Config;
use crate::macos;
use crate::plugins::{self, Output, Plugin};
use crate::tray::{Action, Tray};

const ENABLED: &str = "enabled";
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
        let plugins: Vec<PluginSlot> = plugins::all()
            .into_iter()
            .map(|mut plugin| {
                for flag in plugin.flags() {
                    let on = config.flag(plugin.id(), flag.id).unwrap_or(flag.on);
                    plugin.set_flag(flag.id, on);
                }
                PluginSlot {
                    enabled: config.flag(plugin.id(), ENABLED).unwrap_or(true),
                    plugin,
                    output: Output::default(),
                }
            })
            .collect();
        let tray = Tray::new(
            plugins
                .iter()
                .map(|slot| (slot.plugin.as_ref(), slot.enabled)),
        );
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
        if !slot.enabled {
            return;
        }
        let output = slot.plugin.sample();
        slot.output.segment = output.segment;
        slot.output.tooltip_lines = output.tooltip_lines;
        if let Some(items) = output.menu_items {
            self.tray.set_section(plugin, items);
        }
    }

    fn render_status(&self) {
        let mut enabled = 0;
        let mut segments = Vec::new();
        let mut blocks = Vec::new();
        for slot in &self.plugins {
            if !slot.enabled {
                continue;
            }
            enabled += 1;
            if let Some(segment) = &slot.output.segment {
                segments.push((slot.plugin.prefix(), segment.as_str()));
            }
            if !slot.output.tooltip_lines.is_empty() {
                blocks.push(slot.output.tooltip_lines.join("\n"));
            }
        }
        self.tray.set_title(status_title(&segments, enabled));
        self.tray.set_tooltip(blocks.join("\n\n"));
    }

    pub fn handle_menu(&mut self, id: &MenuId) -> ControlFlow<()> {
        match self.tray.action(id) {
            Some(Action::ToggleFlag(plugin, flag)) => self.toggle_flag(plugin, flag),
            Some(Action::TogglePlugin(plugin)) => self.toggle_plugin(plugin),
            Some(Action::ToggleLogin) => self.toggle_login(),
            Some(Action::RefreshNow) => self.refresh_now(),
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

    fn toggle_plugin(&mut self, plugin: usize) {
        let enabled = !self.plugins[plugin].enabled;
        self.plugins[plugin].enabled = enabled;
        self.config
            .set_flag(self.plugins[plugin].plugin.id(), ENABLED, enabled);
        self.config.save();
        if enabled {
            self.plugins[plugin].plugin.invalidate();
            self.sample_slot(plugin);
        } else {
            self.plugins[plugin].output = Output::default();
            self.tray.set_section(plugin, Vec::new());
        }
        self.render_status();
    }

    fn toggle_login(&mut self) {
        let enabled = macos::set_login_enabled(!macos::login_enabled());
        self.tray.set_login_checked(enabled);
    }

    fn refresh_now(&mut self) {
        for slot in &mut self.plugins {
            slot.plugin.invalidate();
        }
        self.sample_all();
    }
}

struct PluginSlot {
    plugin: Box<dyn Plugin>,
    enabled: bool,
    output: Output,
}

fn status_title(segments: &[(&str, &str)], enabled: usize) -> String {
    if segments.is_empty() {
        "\u{2026}".to_string()
    } else if enabled == 1 {
        segments[0].1.to_string()
    } else {
        segments
            .iter()
            .map(|(prefix, segment)| format!("{prefix}{segment}"))
            .collect::<Vec<_>>()
            .join(" ")
    }
}
