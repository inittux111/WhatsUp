//! The main window: a slim header bar above WhatsApp Web.

use adw::prelude::*;
use gtk::{gio, glib};
use webkit::prelude::*;

use crate::app::APP_NAME;
use crate::{paths, web};

/// The main window and the parts of it that other modules attach to.
pub struct Main {
    pub window: adw::ApplicationWindow,
    pub view: webkit::WebView,
    pub toasts: adw::ToastOverlay,
}

pub fn build(app: &adw::Application) -> std::io::Result<Main> {
    let dirs = paths::account_dirs("default")?;
    let view = web::build(&dirs);

    let header = adw::HeaderBar::new();
    header.pack_end(&main_menu_button());

    let content = adw::ToolbarView::new();
    content.add_top_bar(&header);
    let toasts = adw::ToastOverlay::new();
    toasts.set_child(Some(&view));
    content.set_content(Some(&toasts));

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(APP_NAME)
        .default_width(1100)
        .default_height(760)
        .width_request(360)
        .height_request(400)
        .content(&content)
        .build();
    add_actions(&window, &view);
    Ok(Main {
        window,
        view,
        toasts,
    })
}

fn main_menu_button() -> gtk::MenuButton {
    let menu = gio::Menu::new();
    menu.append(Some("_Reload"), Some("win.reload"));
    menu.append(Some("_About WhatsUp"), Some("win.about"));
    menu.append(Some("_Quit"), Some("app.quit"));
    gtk::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .tooltip_text("Main Menu")
        .menu_model(&menu)
        .primary(true)
        .build()
}

fn add_actions(window: &adw::ApplicationWindow, view: &webkit::WebView) {
    let reload = gio::ActionEntry::builder("reload")
        .activate(glib::clone!(
            #[weak]
            view,
            move |_: &adw::ApplicationWindow, _, _| view.reload()
        ))
        .build();
    let about = gio::ActionEntry::builder("about")
        .activate(|window: &adw::ApplicationWindow, _, _| show_about(window))
        .build();
    window.add_action_entries([reload, about]);
}

fn show_about(window: &adw::ApplicationWindow) {
    adw::AboutDialog::builder()
        .application_name(APP_NAME)
        .application_icon(crate::app::APP_ID)
        .version(env!("CARGO_PKG_VERSION"))
        .comments("WhatsApp Web for GNOME")
        .license_type(gtk::License::Gpl30)
        .build()
        .present(Some(window));
}
