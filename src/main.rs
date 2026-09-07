use std::cell::RefCell;
use std::rc::Rc;

use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS};
use tray_icon::menu::MenuEvent;

use crate::app::App;

mod app;
mod config;
mod format;
mod macos;
mod plugins;
mod tray;

fn main() {
    let mut event_loop = EventLoopBuilder::<MenuEvent>::with_user_event().build();
    event_loop.set_activation_policy(ActivationPolicy::Accessory);

    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(event);
    }));

    let app: Rc<RefCell<Option<App>>> = Rc::new(RefCell::new(None));

    event_loop.run(move |event, _, control_flow| {
        let mut slot = app.borrow_mut();
        match event {
            Event::NewEvents(StartCause::Init) => {
                *slot = Some(App::new(menu_open_handler(Rc::clone(&app))));
            }
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                if let Some(app) = slot.as_mut() {
                    app.tick();
                }
            }
            Event::UserEvent(menu_event) => {
                if let Some(app) = slot.as_mut()
                    && app.handle_menu(menu_event.id()).is_break()
                {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            _ => {}
        }
        *control_flow = match slot.as_ref() {
            Some(app) => ControlFlow::WaitUntil(app.next_tick()),
            None => ControlFlow::Wait,
        };
    });
}

fn menu_open_handler(app: Rc<RefCell<Option<App>>>) -> Box<dyn Fn()> {
    Box::new(move || {
        if let Ok(mut slot) = app.try_borrow_mut()
            && let Some(app) = slot.as_mut()
        {
            app.menu_will_open();
        }
    })
}
