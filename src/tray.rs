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
    title: String,
    tooltip: String,
}

impl Tray {
    pub fn new<'a>(
        plugins: impl Iterator<Item = (&'a dyn Plugin, bool)>,
        on_menu_open: Box<dyn Fn()>,
    ) -> Self {
        let menu = Menu::new();
        let mut actions = HashMap::new();

        let mut sections = Vec::new();
        let settings_menu = Submenu::new("Settings", true);
        for (plugin, (source, enabled)) in plugins.enumerate() {
            let submenu = Submenu::new(source.label(), true);
            let check = CheckMenuItem::new("Enabled", true, enabled, None);
            actions.insert(check.id().clone(), Action::TogglePlugin(plugin));
            let _ = submenu.append(&check);
            let flags = source.flags();
            if !flags.is_empty() {
                let _ = submenu.append(&PredefinedMenuItem::separator());
            }
            for flag in flags {
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

        let refresh = MenuItem::new("Refresh Now", true, None);
        actions.insert(refresh.id().clone(), Action::RefreshNow);

        let quit = MenuItem::new("Quit Recall", true, None);
        actions.insert(quit.id().clone(), Action::Quit);

        let _ = menu.append(&refresh);
        let _ = menu.append(&settings_menu);
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&quit);

        let title = "\u{2026}".to_string();
        let icon = TrayIconBuilder::new()
            .with_title(&title)
            .with_menu(Box::new(menu.clone()))
            .build()
            .expect("create tray icon");
        macos::set_monospaced_font(&icon);
        macos::observe_menu_open(&icon, on_menu_open);

        Self {
            icon,
            menu,
            actions,
            sections,
            rows: Vec::new(),
            login_item,
            title,
            tooltip: String::new(),
        }
    }

    pub fn set_title(&mut self, title: String) {
        if self.title == title {
            return;
        }
        self.icon.set_title(Some(&title));
        self.title = title;
    }

    pub fn set_tooltip(&mut self, tooltip: String) {
        if self.tooltip == tooltip {
            return;
        }
        let _ = self.icon.set_tooltip(Some(&tooltip));
        self.tooltip = tooltip;
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
    TogglePlugin(usize),
    ToggleLogin,
    RefreshNow,
    Quit,
}
