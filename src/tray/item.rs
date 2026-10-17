//! The tray icon itself, over org.kde.StatusNotifierItem, which GNOME shows
//! through the AppIndicator extension.

use gtk::glib::{Variant, prelude::*};

pub const OBJECT_PATH: &str = "/StatusNotifierItem";
pub const INTERFACE: &str = "org.kde.StatusNotifierItem";

pub const XML: &str = r#"<node>
  <interface name="org.kde.StatusNotifierItem">
    <method name="Activate"><arg type="i" direction="in"/><arg type="i" direction="in"/></method>
    <method name="SecondaryActivate"><arg type="i" direction="in"/><arg type="i" direction="in"/></method>
    <method name="ContextMenu"><arg type="i" direction="in"/><arg type="i" direction="in"/></method>
    <method name="Scroll"><arg type="i" direction="in"/><arg type="s" direction="in"/></method>
    <signal name="NewIcon"/>
    <signal name="NewAttentionIcon"/>
    <signal name="NewToolTip"/>
    <signal name="NewStatus"><arg type="s"/></signal>
    <property name="Category" type="s" access="read"/>
    <property name="Id" type="s" access="read"/>
    <property name="Title" type="s" access="read"/>
    <property name="Status" type="s" access="read"/>
    <property name="WindowId" type="i" access="read"/>
    <property name="IconName" type="s" access="read"/>
    <property name="AttentionIconName" type="s" access="read"/>
    <property name="IconThemePath" type="s" access="read"/>
    <property name="ToolTip" type="(sa(iiay)ss)" access="read"/>
    <property name="ItemIsMenu" type="b" access="read"/>
    <property name="Menu" type="o" access="read"/>
  </interface>
</node>"#;

const ICON: &str = "app.whatsup.WhatsUp-symbolic";
const UNREAD_ICON: &str = "app.whatsup.WhatsUp-unread-symbolic";

/// The icon's state: "Passive" hides it, and with unread chats the tray
/// shows the attention icon.
pub fn status(unread: u32, hidden: bool) -> &'static str {
    if hidden {
        "Passive"
    } else if unread > 0 {
        "NeedsAttention"
    } else {
        "Active"
    }
}

pub fn tooltip_text(unread: u32) -> String {
    match unread {
        0 => "No unread chats".to_owned(),
        1 => "1 unread chat".to_owned(),
        n => format!("{n} unread chats"),
    }
}

/// The value of property `name`, for `unread` unread chats.
pub fn property(name: &str, unread: u32, hidden: bool, icon_theme_path: &str) -> Option<Variant> {
    let empty_pixmaps: Vec<(i32, i32, Vec<u8>)> = Vec::new();
    Some(match name {
        "Category" => "Communications".to_variant(),
        "Id" => crate::app::APP_ID.to_variant(),
        "Title" => crate::app::APP_NAME.to_variant(),
        "Status" => status(unread, hidden).to_variant(),
        "WindowId" => 0i32.to_variant(),
        "IconName" => ICON.to_variant(),
        "AttentionIconName" => UNREAD_ICON.to_variant(),
        "IconThemePath" => icon_theme_path.to_variant(),
        "ToolTip" => (
            ICON,
            empty_pixmaps,
            crate::app::APP_NAME,
            tooltip_text(unread),
        )
            .to_variant(),
        "ItemIsMenu" => false.to_variant(),
        "Menu" => gtk::glib::variant::ObjectPath::try_from(super::menu::OBJECT_PATH)
            .ok()?
            .to_variant(),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_and_tooltip_follow_unread() {
        assert_eq!(status(0, false), "Active");
        assert_eq!(status(2, false), "NeedsAttention");
        assert_eq!(status(2, true), "Passive");
        assert_eq!(tooltip_text(0), "No unread chats");
        assert_eq!(tooltip_text(1), "1 unread chat");
        assert_eq!(tooltip_text(5), "5 unread chats");
    }

    #[test]
    fn properties_have_their_dbus_types() {
        let ty = |name| {
            property(name, 1, false, "")
                .unwrap()
                .type_()
                .as_str()
                .to_owned()
        };
        assert_eq!(ty("ToolTip"), "(sa(iiay)ss)");
        assert_eq!(ty("Menu"), "o");
        assert_eq!(ty("WindowId"), "i");
        assert!(property("Nonexistent", 0, false, "").is_none());
    }
}
