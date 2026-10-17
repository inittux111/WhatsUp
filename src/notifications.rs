//! WhatsApp Web's notifications as GNOME notifications, so they follow the
//! desktop's Do Not Disturb and per-app settings. Clicking one brings the
//! window to the front and lets WhatsApp open that chat.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use adw::prelude::*;
use gtk::{gio, glib};
use webkit::prelude::*;

const OPEN_ACTION: &str = "open-notification";

/// The notifications on screen, by their GNotification id.
type Shown = Rc<RefCell<HashMap<String, webkit::Notification>>>;

pub fn attach(app: &adw::Application, window: &adw::ApplicationWindow, view: &webkit::WebView) {
    let shown = Shown::default();

    let open = gio::SimpleAction::new(OPEN_ACTION, Some(glib::VariantTy::STRING));
    open.connect_activate(glib::clone!(
        #[weak]
        window,
        #[strong]
        shown,
        move |_, param| {
            window.present();
            let id = param.and_then(|p| p.str().map(str::to_owned));
            let notification = id.and_then(|id| shown.borrow_mut().remove(&id));
            if let Some(notification) = notification {
                notification.clicked();
            }
        }
    ));
    app.add_action(&open);

    view.connect_show_notification(glib::clone!(
        #[weak]
        app,
        #[upgrade_or]
        false,
        move |_, notification| {
            show(&app, &shown, notification);
            true
        }
    ));
}

fn show(app: &adw::Application, shown: &Shown, notification: &webkit::Notification) {
    let id = format!("message-{}", notification.id());
    let title = notification.title().unwrap_or_default();
    let desktop = gio::Notification::new(&title);
    if let Some(body) = notification.body().filter(|body| !body.is_empty()) {
        desktop.set_body(Some(&body));
    }
    desktop.set_category(Some("im.received"));
    desktop
        .set_default_action_and_target_value(&format!("app.{OPEN_ACTION}"), Some(&id.to_variant()));

    notification.connect_closed(glib::clone!(
        #[weak]
        app,
        #[strong]
        shown,
        #[strong]
        id,
        move |_| {
            shown.borrow_mut().remove(&id);
            app.withdraw_notification(&id);
        }
    ));
    shown.borrow_mut().insert(id.clone(), notification.clone());
    app.send_notification(Some(&id), &desktop);
}
