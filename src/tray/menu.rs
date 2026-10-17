//! The tray icon's menu, over com.canonical.dbusmenu: Show/Hide and Quit.

use gtk::glib::{Variant, prelude::*};
use std::collections::HashMap;

pub const OBJECT_PATH: &str = "/MenuBar";
pub const INTERFACE: &str = "com.canonical.dbusmenu";

pub const XML: &str = r#"<node>
  <interface name="com.canonical.dbusmenu">
    <method name="GetLayout">
      <arg type="i" direction="in"/><arg type="i" direction="in"/><arg type="as" direction="in"/>
      <arg type="u" direction="out"/><arg type="(ia{sv}av)" direction="out"/>
    </method>
    <method name="GetGroupProperties">
      <arg type="ai" direction="in"/><arg type="as" direction="in"/>
      <arg type="a(ia{sv})" direction="out"/>
    </method>
    <method name="GetProperty">
      <arg type="i" direction="in"/><arg type="s" direction="in"/><arg type="v" direction="out"/>
    </method>
    <method name="Event">
      <arg type="i" direction="in"/><arg type="s" direction="in"/>
      <arg type="v" direction="in"/><arg type="u" direction="in"/>
    </method>
    <method name="EventGroup">
      <arg type="a(isvu)" direction="in"/><arg type="ai" direction="out"/>
    </method>
    <method name="AboutToShow">
      <arg type="i" direction="in"/><arg type="b" direction="out"/>
    </method>
    <method name="AboutToShowGroup">
      <arg type="ai" direction="in"/><arg type="ai" direction="out"/><arg type="ai" direction="out"/>
    </method>
    <signal name="LayoutUpdated"><arg type="u"/><arg type="i"/></signal>
    <signal name="ItemsPropertiesUpdated">
      <arg type="a(ia{sv})"/><arg type="a(ias)"/>
    </signal>
    <property name="Version" type="u" access="read"/>
    <property name="TextDirection" type="s" access="read"/>
    <property name="Status" type="s" access="read"/>
    <property name="IconThemePath" type="as" access="read"/>
  </interface>
</node>"#;

/// The menu's entries; their ids are what `Event` reports back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    ShowHide = 1,
    Quit = 3,
}

const SEPARATOR: i32 = 2;

impl Entry {
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            1 => Some(Self::ShowHide),
            3 => Some(Self::Quit),
            _ => None,
        }
    }
}

/// The whole menu as dbusmenu's `(ia{sv}av)` layout.
pub fn layout(window_visible: bool) -> Variant {
    let show_hide = if window_visible {
        "Hide WhatsUp"
    } else {
        "Show WhatsUp"
    };
    let children = vec![
        item(Entry::ShowHide as i32, &[("label", show_hide.to_variant())]),
        item(SEPARATOR, &[("type", "separator".to_variant())]),
        item(Entry::Quit as i32, &[("label", "Quit".to_variant())]),
    ];
    node(0, &[("children-display", "submenu".to_variant())], children)
}

fn item(id: i32, props: &[(&str, Variant)]) -> Variant {
    node(id, props, Vec::new())
}

fn node(id: i32, props: &[(&str, Variant)], children: Vec<Variant>) -> Variant {
    let props: HashMap<&str, Variant> = props.iter().cloned().collect();
    (id, props, children).to_variant()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_has_the_dbusmenu_type() {
        assert_eq!(layout(true).type_().as_str(), "(ia{sv}av)");
    }

    #[test]
    fn children_are_boxed_once() {
        let child = layout(true).child_value(2).child_value(0);
        assert_eq!(child.type_().as_str(), "v");
        assert_eq!(child.as_variant().unwrap().type_().as_str(), "(ia{sv}av)");
    }

    #[test]
    fn label_follows_the_window() {
        let text = |visible| layout(visible).print(false).to_string();
        assert!(text(true).contains("Hide WhatsUp"));
        assert!(text(false).contains("Show WhatsUp"));
    }

    #[test]
    fn entries_round_trip() {
        assert_eq!(
            Entry::from_id(Entry::ShowHide as i32),
            Some(Entry::ShowHide)
        );
        assert_eq!(Entry::from_id(Entry::Quit as i32), Some(Entry::Quit));
        assert_eq!(Entry::from_id(SEPARATOR), None);
    }
}
