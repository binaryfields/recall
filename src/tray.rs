use std::collections::HashMap;

use tray_icon::menu::{CheckMenuItem, Menu, MenuId, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{TrayIcon, TrayIconBuilder};

use crate::config::{MemoryMetric, Readout};
use crate::format;
use crate::macos;
use crate::stats::ProcInfo;

pub struct Tray {
    icon: TrayIcon,
    menu: Menu,
    actions: HashMap<MenuId, Action>,
    process_items: Vec<MenuItem>,
    format_toggle: MenuItem,
    memory_toggle: MenuItem,
    login_item: CheckMenuItem,
}

impl Tray {
    pub fn new(readout: Readout, memory: MemoryMetric) -> Self {
        let menu = Menu::new();
        let mut actions = HashMap::new();

        let format_toggle = MenuItem::new(readout_label(readout), true, None);
        actions.insert(format_toggle.id().clone(), Action::ToggleFormat);
        let memory_toggle = MenuItem::new(metric_label(memory), true, None);
        actions.insert(memory_toggle.id().clone(), Action::ToggleMemory);
        let login_item = CheckMenuItem::new("Start at Login", true, macos::login_enabled(), None);
        actions.insert(login_item.id().clone(), Action::ToggleLogin);
        let quit = MenuItem::new("Quit Recall", true, None);
        actions.insert(quit.id().clone(), Action::Quit);

        let settings = Submenu::new("Settings", true);
        let _ = settings.append(&format_toggle);
        let _ = settings.append(&memory_toggle);
        let _ = settings.append(&login_item);

        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&settings);
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&quit);

        let icon = TrayIconBuilder::new()
            .with_title("\u{2026}")
            .with_menu(Box::new(menu.clone()))
            .build()
            .expect("create tray icon");
        macos::set_monospaced_font(&icon);

        Self {
            icon,
            menu,
            actions,
            process_items: Vec::new(),
            format_toggle,
            memory_toggle,
            login_item,
        }
    }

    pub fn set_title(&self, title: String) {
        self.icon.set_title(Some(title));
    }

    pub fn set_tooltip(&self, tooltip: String) {
        let _ = self.icon.set_tooltip(Some(tooltip));
    }

    pub fn set_processes(&mut self, procs: &[ProcInfo]) {
        for item in self.process_items.drain(..) {
            let _ = self.menu.remove(&item);
        }

        let mut badges = Vec::with_capacity(procs.len());
        for (index, proc) in procs.iter().enumerate() {
            let item = MenuItem::new(&proc.name, true, None);
            let _ = self.menu.insert(&item, index);
            self.process_items.push(item);
            badges.push(format::scaled(proc.memory));
        }
        macos::set_process_badges(&self.icon, &badges);
    }

    pub fn action(&self, id: &MenuId) -> Option<&Action> {
        self.actions.get(id)
    }

    pub fn set_login_checked(&self, checked: bool) {
        self.login_item.set_checked(checked);
    }

    pub fn set_readout(&self, readout: Readout) {
        self.format_toggle.set_text(readout_label(readout));
    }

    pub fn set_memory(&self, memory: MemoryMetric) {
        self.memory_toggle.set_text(metric_label(memory));
    }
}

pub enum Action {
    ToggleFormat,
    ToggleMemory,
    ToggleLogin,
    Quit,
}

fn readout_label(readout: Readout) -> &'static str {
    match readout {
        Readout::Percent => "Show Absolute Usage",
        Readout::Absolute => "Show Percentage",
    }
}

fn metric_label(memory: MemoryMetric) -> &'static str {
    match memory {
        MemoryMetric::Footprint => "Show RSS Memory",
        MemoryMetric::Rss => "Show Footprint Memory",
    }
}
