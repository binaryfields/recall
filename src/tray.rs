use std::collections::HashMap;

use tray_icon::menu::{
    CheckMenuItem, IsMenuItem, Menu, MenuId, MenuItem, PredefinedMenuItem, Submenu,
};
use tray_icon::{TrayIcon, TrayIconBuilder};

use crate::macos;
use crate::plugins::{Item, Plugin};

pub struct Tray {
    icon: TrayIcon,
    menu: Menu,
    actions: HashMap<MenuId, Action>,
    sections: Vec<Vec<Item>>,
    rows: Vec<Box<dyn IsMenuItem>>,
    login_item: CheckMenuItem,
}

impl Tray {
    pub fn new(plugins: &[Box<dyn Plugin>]) -> Self {
        let menu = Menu::new();
        let mut actions = HashMap::new();

        let mut sections = Vec::new();
        let settings_menu = Submenu::new("Settings", true);
        for (plugin, source) in plugins.iter().enumerate() {
            let submenu = Submenu::new(source.label(), true);
            for flag in source.flags() {
                let check = CheckMenuItem::new(flag.label, true, flag.on, None);
                actions.insert(check.id().clone(), Action::ToggleFlag(plugin, flag.id));
                let _ = submenu.append(&check);
            }
            let _ = settings_menu.append(&submenu);
            sections.push(Vec::new());
        }
        let _ = settings_menu.append(&PredefinedMenuItem::separator());
        let login_item = CheckMenuItem::new("Start at Login", true, macos::login_enabled(), None);
        actions.insert(login_item.id().clone(), Action::ToggleLogin);
        let _ = settings_menu.append(&login_item);

        let quit = MenuItem::new("Quit Recall", true, None);
        actions.insert(quit.id().clone(), Action::Quit);

        let _ = menu.append(&settings_menu);
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
            sections,
            rows: Vec::new(),
            login_item,
        }
    }

    pub fn set_title(&self, title: String) {
        self.icon.set_title(Some(title));
    }

    pub fn set_tooltip(&self, tooltip: String) {
        let _ = self.icon.set_tooltip(Some(tooltip));
    }

    pub fn set_section(&mut self, plugin: usize, items: Vec<Item>) {
        if self.sections[plugin] == items {
            return;
        }
        self.sections[plugin] = items;
        for row in self.rows.drain(..) {
            let _ = self.menu.remove(row.as_ref());
        }
        let mut badges = Vec::new();
        for items in self.sections.iter().filter(|items| !items.is_empty()) {
            for item in items {
                badges.push(item.badge.as_deref());
                self.rows
                    .push(Box::new(MenuItem::new(&item.text, true, None)));
            }
            badges.push(None);
            self.rows.push(Box::new(PredefinedMenuItem::separator()));
        }
        for (row, menu_item) in self.rows.iter().enumerate() {
            let _ = self.menu.insert(menu_item.as_ref(), row);
        }
        macos::set_menu_badges(&self.icon, &badges);
    }

    pub fn action(&self, id: &MenuId) -> Option<Action> {
        self.actions.get(id).copied()
    }

    pub fn set_login_checked(&self, checked: bool) {
        self.login_item.set_checked(checked);
    }
}

#[derive(Clone, Copy)]
pub enum Action {
    ToggleFlag(usize, &'static str),
    ToggleLogin,
    Quit,
}
