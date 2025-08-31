use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS};
use tray_icon::menu::MenuEvent;

use crate::app::App;

mod app;
mod format;
mod macos;
mod stats;
mod tray;

fn main() {
    let mut event_loop = EventLoopBuilder::<MenuEvent>::with_user_event().build();
    event_loop.set_activation_policy(ActivationPolicy::Accessory);

    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(event);
    }));

    let mut app: Option<App> = None;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::NewEvents(StartCause::Init) => app = Some(App::new()),
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                if let Some(app) = app.as_mut() {
                    app.tick();
                }
            }
            Event::UserEvent(menu_event) => {
                if let Some(app) = app.as_mut()
                    && app.dispatch(menu_event.id())
                {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            _ => {}
        }
        *control_flow = match app.as_ref() {
            Some(app) => ControlFlow::WaitUntil(app.next_tick()),
            None => ControlFlow::Wait,
        };
    });
}
