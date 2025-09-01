use std::collections::HashMap;

use tray_icon::menu::{Menu, MenuId, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{TrayIcon, TrayIconBuilder};

use crate::config::Readout;
use crate::macos;

pub struct Tray {
    icon: TrayIcon,
    actions: HashMap<MenuId, Action>,
    format_toggle: MenuItem,
}

impl Tray {
    pub fn new(readout: Readout) -> Self {
        let menu = Menu::new();
        let mut actions = HashMap::new();

        let format_toggle = MenuItem::new(readout_label(readout), true, None);
        actions.insert(format_toggle.id().clone(), Action::ToggleFormat);
        let quit = MenuItem::new("Quit Recall", true, None);
        actions.insert(quit.id().clone(), Action::Quit);

        let settings = Submenu::new("Settings", true);
        let _ = settings.append(&format_toggle);

        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&settings);
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&quit);

        let icon = TrayIconBuilder::new()
            .with_title("\u{2026}")
            .with_menu(Box::new(menu))
            .build()
            .expect("create tray icon");
        macos::set_monospaced_font(&icon);

        Self {
            icon,
            actions,
            format_toggle,
        }
    }

    pub fn set_title(&self, title: String) {
        self.icon.set_title(Some(title));
    }

    pub fn set_tooltip(&self, tooltip: String) {
        let _ = self.icon.set_tooltip(Some(tooltip));
    }

    pub fn action(&self, id: &MenuId) -> Option<&Action> {
        self.actions.get(id)
    }

    pub fn set_readout(&self, readout: Readout) {
        self.format_toggle.set_text(readout_label(readout));
    }
}

pub enum Action {
    ToggleFormat,
    Quit,
}

fn readout_label(readout: Readout) -> &'static str {
    match readout {
        Readout::Percent => "Show Absolute Usage",
        Readout::Absolute => "Show Percentage",
    }
}
