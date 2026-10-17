//! The tray icon: shows whether there are unread chats, toggles the window
//! on click and offers Show/Hide and Quit in its menu. It registers with the
//! desktop's StatusNotifierWatcher, again whenever the watcher restarts.

mod item;
mod menu;

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use gtk::gio;
use gtk::glib::{self, Variant, prelude::*};

const WATCHER: &str = "org.kde.StatusNotifierWatcher";

/// What the icon does; each runs from the main loop, never inside a D-Bus call.
pub struct Actions {
    /// Shows or hides the window: a click on the icon, or Show/Hide.
    pub toggle: Box<dyn Fn()>,
    pub settings: Box<dyn Fn()>,
    pub quit: Box<dyn Fn()>,
}

pub struct Tray {
    inner: Rc<Inner>,
}

struct Inner {
    connection: gio::DBusConnection,
    icon_theme_path: String,
    state: RefCell<State>,
    actions: Actions,
}

#[derive(Default)]
struct State {
    unread: u32,
    hidden: bool,
    /// The green icon rather than the symbolic one.
    colored: bool,
    window_visible: bool,
    revision: u32,
}

impl Tray {
    /// Puts the icon on `connection`; its clicks and menu run `actions`.
    pub fn new(
        connection: &gio::DBusConnection,
        icon_theme_path: String,
        actions: Actions,
    ) -> Result<Self, glib::Error> {
        let inner = Rc::new(Inner {
            connection: connection.clone(),
            icon_theme_path,
            state: RefCell::default(),
            actions,
        });
        register_item(&inner)?;
        register_menu(&inner)?;
        watch_watcher(&inner);
        Ok(Self { inner })
    }

    pub fn set_unread(&self, unread: u32) {
        let changed = {
            let mut state = self.inner.state.borrow_mut();
            std::mem::replace(&mut state.unread, unread) != unread
        };
        if changed {
            self.announce();
        }
    }

    /// Shows or hides the icon; a hidden icon stays registered, so showing
    /// it again is instant.
    pub fn set_icon_visible(&self, visible: bool) {
        let changed = {
            let mut state = self.inner.state.borrow_mut();
            std::mem::replace(&mut state.hidden, !visible) == visible
        };
        if changed {
            self.announce();
        }
    }

    /// Shows the green icon, or the symbolic one that follows the theme.
    pub fn set_colored(&self, colored: bool) {
        let changed = {
            let mut state = self.inner.state.borrow_mut();
            std::mem::replace(&mut state.colored, colored) != colored
        };
        if changed {
            self.announce();
            self.inner.emit_item("NewAttentionIcon", None);
        }
    }

    fn announce(&self) {
        let status = {
            let state = self.inner.state.borrow();
            item::status(state.unread, state.hidden)
        };
        self.inner
            .emit_item("NewStatus", Some((status,).to_variant()));
        self.inner.emit_item("NewIcon", None);
        self.inner.emit_item("NewToolTip", None);
    }

    pub fn set_window_visible(&self, visible: bool) {
        let revision = {
            let mut state = self.inner.state.borrow_mut();
            if state.window_visible == visible {
                return;
            }
            state.window_visible = visible;
            state.revision += 1;
            state.revision
        };
        let params = (revision, 0i32).to_variant();
        self.inner.emit(
            menu::OBJECT_PATH,
            menu::INTERFACE,
            "LayoutUpdated",
            Some(params),
        );
    }
}

impl Inner {
    fn emit_item(&self, signal: &str, params: Option<Variant>) {
        self.emit(item::OBJECT_PATH, item::INTERFACE, signal, params);
    }

    fn emit(&self, path: &str, interface: &str, signal: &str, params: Option<Variant>) {
        if let Err(err) =
            self.connection
                .emit_signal(None, path, interface, signal, params.as_ref())
        {
            glib::g_debug!(
                "whatsup-desktop",
                "tray signal {signal} failed: {}",
                err.message()
            );
        }
    }

    /// Runs `f` on `inner` from the main loop, after the D-Bus call returned.
    fn later(inner: &Weak<Inner>, f: fn(&Inner)) {
        let inner = inner.clone();
        glib::idle_add_local_once(move || {
            if let Some(inner) = inner.upgrade() {
                f(&inner);
            }
        });
    }
}

fn interface_info(xml: &str, name: &str) -> Result<gio::DBusInterfaceInfo, glib::Error> {
    gio::DBusNodeInfo::for_xml(xml)?
        .lookup_interface(name)
        .ok_or_else(|| glib::Error::new(gio::IOErrorEnum::NotFound, "missing D-Bus interface"))
}

fn register_item(inner: &Rc<Inner>) -> Result<(), glib::Error> {
    let info = interface_info(item::XML, item::INTERFACE)?;
    let (for_calls, for_props) = (Rc::downgrade(inner), Rc::downgrade(inner));
    inner
        .connection
        .register_object(item::OBJECT_PATH, &info)
        .method_call(move |_, _, _, _, method, _, invocation| {
            if matches!(method, "Activate" | "SecondaryActivate") {
                Inner::later(&for_calls, |inner| (inner.actions.toggle)());
            }
            invocation.return_value(None);
        })
        .property(move |_, _, _, _, name| {
            let Some(inner) = for_props.upgrade() else {
                return "".to_variant();
            };
            let (unread, hidden, colored) = {
                let state = inner.state.borrow();
                (state.unread, state.hidden, state.colored)
            };
            item::property(name, unread, hidden, colored, &inner.icon_theme_path)
                .unwrap_or_else(|| "".to_variant())
        })
        .build()?;
    Ok(())
}

fn register_menu(inner: &Rc<Inner>) -> Result<(), glib::Error> {
    let info = interface_info(menu::XML, menu::INTERFACE)?;
    let (for_calls, for_props) = (Rc::downgrade(inner), Rc::downgrade(inner));
    inner
        .connection
        .register_object(menu::OBJECT_PATH, &info)
        .method_call(move |_, _, _, _, method, params, invocation| {
            let reply = menu_call(&for_calls, method, &params);
            invocation.return_value(reply.as_ref());
        })
        .property(move |_, _, _, _, name| {
            let path = for_props.upgrade().map(|i| i.icon_theme_path.clone());
            match name {
                "Version" => 3u32.to_variant(),
                "TextDirection" => "ltr".to_variant(),
                "Status" => "normal".to_variant(),
                _ => path.into_iter().collect::<Vec<_>>().to_variant(),
            }
        })
        .build()?;
    Ok(())
}

/// Answers one dbusmenu call; the reply is the method's out arguments.
fn menu_call(inner: &Weak<Inner>, method: &str, params: &Variant) -> Option<Variant> {
    let no_ids: Vec<i32> = Vec::new();
    match method {
        "GetLayout" => {
            let state = inner.upgrade()?;
            let state = state.state.borrow();
            Some(Variant::tuple_from_iter([
                state.revision.to_variant(),
                menu::layout(state.window_visible),
            ]))
        }
        "GetGroupProperties" => {
            let none: Vec<(i32, std::collections::HashMap<String, Variant>)> = Vec::new();
            Some((none,).to_variant())
        }
        "GetProperty" => Some((Variant::from_variant(&"".to_variant()),).to_variant()),
        "Event" => {
            if let Some((id, event, _, _)) = params.get::<(i32, String, Variant, u32)>() {
                menu_event(inner, id, &event);
            }
            None
        }
        "EventGroup" => {
            let events = params.get::<(Vec<(i32, String, Variant, u32)>,)>();
            for (id, event, _, _) in events.map(|(e,)| e).unwrap_or_default() {
                menu_event(inner, id, &event);
            }
            Some((no_ids,).to_variant())
        }
        "AboutToShow" => Some((false,).to_variant()),
        "AboutToShowGroup" => Some((no_ids.clone(), no_ids).to_variant()),
        _ => None,
    }
}

fn menu_event(inner: &Weak<Inner>, id: i32, event: &str) {
    if event != "clicked" {
        return;
    }
    match menu::Entry::from_id(id) {
        Some(menu::Entry::ShowHide) => Inner::later(inner, |inner| (inner.actions.toggle)()),
        Some(menu::Entry::Settings) => Inner::later(inner, |inner| (inner.actions.settings)()),
        Some(menu::Entry::Quit) => Inner::later(inner, |inner| (inner.actions.quit)()),
        None => {}
    }
}

/// Registers the icon whenever a StatusNotifierWatcher appears on the bus.
fn watch_watcher(inner: &Rc<Inner>) {
    let weak = Rc::downgrade(inner);
    // The watch lasts as long as the app; the id is deliberately never unwatched.
    let _ = gio::bus_watch_name_on_connection(
        &inner.connection,
        WATCHER,
        gio::BusNameWatcherFlags::NONE,
        move |connection, _, _| {
            if weak.upgrade().is_some() {
                glib::spawn_future_local(register_with_watcher(connection));
            }
        },
        |_, _| {},
    );
}

async fn register_with_watcher(connection: gio::DBusConnection) {
    let result = connection
        .call_future(
            Some(WATCHER),
            "/StatusNotifierWatcher",
            WATCHER,
            "RegisterStatusNotifierItem",
            Some(&(item::OBJECT_PATH,).to_variant()),
            None,
            gio::DBusCallFlags::NONE,
            -1,
        )
        .await;
    if let Err(err) = result {
        glib::g_warning!(
            "whatsup-desktop",
            "cannot show the tray icon: {}",
            err.message()
        );
    }
}
