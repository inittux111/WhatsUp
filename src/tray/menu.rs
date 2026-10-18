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
    Settings = 4,
}

const SEPARATOR: i32 = 2;

impl Entry {
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            1 => Some(Self::ShowHide),
            3 => Some(Self::Quit),
            4 => Some(Self::Settings),
            _ => None,
        }
    }
}

const ROOT: i32 = 0;

type Props = Vec<(&'static str, Variant)>;

/// The root's properties and its children's ids and properties.
fn entries(window_visible: bool) -> (Props, Vec<(i32, Props)>) {
    let show_hide = if window_visible {
        "Hide WhatsUp"
    } else {
        "Show WhatsUp"
    };
    let root = vec![("children-display", "submenu".to_variant())];
    let children = vec![
        (
            Entry::ShowHide as i32,
            vec![("label", show_hide.to_variant())],
        ),
        (
            Entry::Settings as i32,
            vec![("label", "Settings".to_variant())],
        ),
        (SEPARATOR, vec![("type", "separator".to_variant())]),
        (Entry::Quit as i32, vec![("label", "Quit".to_variant())]),
    ];
    (root, children)
}

/// The menu as dbusmenu's `(ia{sv}av)` layout, from item `id` (0 for the
/// whole menu) down, or `None` when there's no such item.
pub fn subtree(window_visible: bool, id: i32) -> Option<Variant> {
    let (root, children) = entries(window_visible);
    if id == ROOT {
        let children = children
            .iter()
            .map(|(id, props)| node(*id, props, Vec::new()));
        return Some(node(ROOT, &root, children.collect()));
    }
    let (id, props) = children.into_iter().find(|(child, _)| *child == id)?;
    Some(node(id, &props, Vec::new()))
}

/// The properties of items `ids` (all items when empty), limited to `names`
/// (all when empty), as dbusmenu's `a(ia{sv})`.
pub fn group_properties(window_visible: bool, ids: &[i32], names: &[String]) -> Variant {
    let (root, children) = entries(window_visible);
    let items = std::iter::once((ROOT, root)).chain(children);
    let wanted = items.filter(|(id, _)| ids.is_empty() || ids.contains(id));
    let groups: Vec<(i32, HashMap<String, Variant>)> = wanted
        .map(|(id, props)| {
            let props = props
                .into_iter()
                .filter(|(name, _)| names.is_empty() || names.iter().any(|n| n == name))
                .map(|(name, value)| (name.to_owned(), value))
                .collect();
            (id, props)
        })
        .collect();
    groups.to_variant()
}

/// Property `name` of item `id`, if it has one.
pub fn property(window_visible: bool, id: i32, name: &str) -> Option<Variant> {
    let (root, children) = entries(window_visible);
    let props = if id == ROOT {
        root
    } else {
        children.into_iter().find(|(child, _)| *child == id)?.1
    };
    props.into_iter().find(|(n, _)| *n == name).map(|(_, v)| v)
}

fn node(id: i32, props: &[(&str, Variant)], children: Vec<Variant>) -> Variant {
    let props: HashMap<&str, Variant> = props.iter().cloned().collect();
    (id, props, children).to_variant()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout(window_visible: bool) -> Variant {
        subtree(window_visible, ROOT).unwrap()
    }

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
    fn subtree_finds_items() {
        assert_eq!(
            subtree(true, ROOT).unwrap().child_value(0).get(),
            Some(ROOT)
        );
        let quit = subtree(true, Entry::Quit as i32).unwrap();
        assert_eq!(quit.child_value(0).get(), Some(Entry::Quit as i32));
        assert_eq!(quit.child_value(2).n_children(), 0);
        assert!(subtree(true, 99).is_none());
    }

    #[test]
    fn group_properties_filter_ids_and_names() {
        let all = group_properties(true, &[], &[]);
        assert_eq!(all.type_().as_str(), "a(ia{sv})");
        assert_eq!(all.n_children(), 5);
        let ids = [Entry::ShowHide as i32, SEPARATOR];
        let some = group_properties(false, &ids, &["label".to_owned()]);
        assert_eq!(some.n_children(), 2);
        assert!(some.print(false).contains("Show WhatsUp"));
        assert!(!some.print(false).contains("separator"));
    }

    #[test]
    fn property_reads_one_value() {
        let label = property(false, Entry::ShowHide as i32, "label").unwrap();
        assert_eq!(label.str(), Some("Show WhatsUp"));
        assert!(property(false, Entry::Quit as i32, "type").is_none());
        assert!(property(false, 99, "label").is_none());
    }

    #[test]
    fn entries_round_trip() {
        assert_eq!(
            Entry::from_id(Entry::ShowHide as i32),
            Some(Entry::ShowHide)
        );
        assert_eq!(Entry::from_id(Entry::Quit as i32), Some(Entry::Quit));
        assert_eq!(
            Entry::from_id(Entry::Settings as i32),
            Some(Entry::Settings)
        );
        assert_eq!(Entry::from_id(SEPARATOR), None);
    }
}
