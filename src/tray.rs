use std::collections::HashMap;

use tray_icon::menu::{Menu, MenuId, MenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder};

use crate::macos;

pub struct Tray {
    icon: TrayIcon,
    actions: HashMap<MenuId, Action>,
}

impl Tray {
    pub fn new() -> Self {
        let menu = Menu::new();
        let mut actions = HashMap::new();

        let quit = MenuItem::new("Quit Recall", true, None);
        actions.insert(quit.id().clone(), Action::Quit);
        let _ = menu.append(&quit);

        let icon = TrayIconBuilder::new()
            .with_title("\u{2026}")
            .with_menu(Box::new(menu))
            .build()
            .expect("create tray icon");
        macos::set_monospaced_font(&icon);

        Self { icon, actions }
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
}

pub enum Action {
    Quit,
}
