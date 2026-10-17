//! The main window: a slim header bar above WhatsApp Web.

use adw::prelude::*;
use gtk::{gio, glib};
use webkit::prelude::*;

use crate::app::APP_NAME;
use crate::settings::Settings;
use crate::{paths, preferences, shortcuts, web};

/// The main window and the parts of it that other modules attach to.
pub struct Main {
    pub window: adw::ApplicationWindow,
    pub view: webkit::WebView,
    pub toasts: adw::ToastOverlay,
    pub settings: Settings,
}

pub fn build(app: &adw::Application, settings: Settings) -> std::io::Result<Main> {
    let dirs = paths::account_dirs("default")?;
    let view = web::build(&dirs);
    web::follow_settings(&view, &settings);

    let header = adw::HeaderBar::new();
    header.pack_end(&main_menu_button(&settings));

    let content = adw::ToolbarView::new();
    content.add_top_bar(&header);
    let toasts = adw::ToastOverlay::new();
    toasts.set_child(Some(&view));
    content.set_content(Some(&toasts));

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(APP_NAME)
        .default_width(settings.get().window.width)
        .default_height(settings.get().window.height)
        .maximized(settings.get().window.maximized)
        .width_request(360)
        .height_request(400)
        .content(&content)
        .build();
    add_actions(&window, &view, &settings);
    window.connect_close_request(glib::clone!(
        #[strong]
        settings,
        move |window| {
            remember_size(window, &settings);
            glib::Propagation::Proceed
        }
    ));
    shortcuts::follow(app, &window, &settings);
    Ok(Main {
        window,
        view,
        toasts,
        settings,
    })
}

/// The ☰ menu; it follows the "Show Zoom in Menu" setting.
/// Stores the window's size, so the next start opens it the same way.
pub fn remember_size(window: &adw::ApplicationWindow, settings: &Settings) {
    let (width, height) = window.default_size();
    let maximized = window.is_maximized();
    settings.update(|v| {
        v.window.maximized = maximized;
        // A maximized window keeps the size it had before, for unmaximizing.
        if !maximized && width > 0 && height > 0 {
            v.window.width = width;
            v.window.height = height;
        }
    });
}

fn main_menu_button(settings: &Settings) -> gtk::MenuButton {
    let button = gtk::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .tooltip_text("Main Menu")
        .menu_model(&main_menu(settings.get().zoom_in_menu))
        .primary(true)
        .build();
    let shown = std::cell::Cell::new(settings.get().zoom_in_menu);
    settings.connect_changed(glib::clone!(
        #[weak]
        button,
        move |values| {
            if shown.replace(values.zoom_in_menu) != values.zoom_in_menu {
                button.set_menu_model(Some(&main_menu(values.zoom_in_menu)));
            }
        }
    ));
    button
}

fn main_menu(with_zoom: bool) -> gio::Menu {
    let zoom = gio::Menu::new();
    zoom.append(Some("Zoom _Out"), Some("win.zoom-out"));
    zoom.append(Some("_Reset Zoom"), Some("win.zoom-reset"));
    zoom.append(Some("Zoom _In"), Some("win.zoom-in"));
    let page = gio::Menu::new();
    page.append(Some("_Reload"), Some("win.reload"));
    let app = gio::Menu::new();
    app.append(Some("_Settings"), Some("win.preferences"));
    app.append(Some("_Keyboard Shortcuts"), Some("win.shortcuts"));
    app.append(Some("_About WhatsUp"), Some("win.about"));
    app.append(Some("_Quit"), Some("app.quit"));
    let menu = gio::Menu::new();
    if with_zoom {
        menu.append_section(None, &zoom);
    }
    menu.append_section(None, &page);
    menu.append_section(None, &app);
    menu
}

fn add_actions(window: &adw::ApplicationWindow, view: &webkit::WebView, settings: &Settings) {
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
    let preferences = gio::ActionEntry::builder("preferences")
        .activate(glib::clone!(
            #[strong]
            settings,
            move |window: &adw::ApplicationWindow, _, _| preferences::show(window, &settings)
        ))
        .build();
    let shortcuts = gio::ActionEntry::builder("shortcuts")
        .activate(glib::clone!(
            #[strong]
            settings,
            move |window: &adw::ApplicationWindow, _, _| shortcuts::show(window, &settings.get())
        ))
        .build();
    let close = gio::ActionEntry::builder("close")
        .activate(|window: &adw::ApplicationWindow, _, _| window.close())
        .build();
    let zoom = |name: &str, change: fn(f64) -> f64| {
        let settings = settings.clone();
        gio::ActionEntry::builder(name)
            .activate(move |_: &adw::ApplicationWindow, _, _| {
                settings.update(|values| values.zoom = change(values.zoom));
            })
            .build()
    };
    let zoom_in = zoom("zoom-in", |z| (z * 10.0 + 1.0).round() / 10.0);
    let zoom_out = zoom("zoom-out", |z| (z * 10.0 - 1.0).round() / 10.0);
    let zoom_reset = zoom("zoom-reset", |_| 1.0);
    window.add_action_entries([
        reload,
        about,
        preferences,
        shortcuts,
        close,
        zoom_in,
        zoom_out,
        zoom_reset,
    ]);
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
