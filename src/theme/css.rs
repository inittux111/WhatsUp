//! Turns the system theme's colours into a stylesheet for WhatsApp Web,
//! which takes its colours from its design system's `--WDS-*` variables.
//! Pure, so it's tested without a display.

/// An RGBA colour, each channel from 0 to 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);

    pub const fn rgb(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// This colour painted over the opaque `base`.
    fn over(self, base: Self) -> Self {
        base.mix(Self { a: 1.0, ..self }, self.a)
    }

    /// `self` moved towards `other` by `t` (0 is `self`, 1 is `other`).
    fn mix(self, other: Self, t: f64) -> Self {
        let t = t.clamp(0.0, 1.0);
        let lerp = |a: f64, b: f64| a + (b - a) * t;
        Self::rgb(
            lerp(self.r, other.r),
            lerp(self.g, other.g),
            lerp(self.b, other.b),
        )
    }

    fn with_alpha(self, a: f64) -> Self {
        Self { a, ..self }
    }

    /// Relative luminance, to tell light themes from dark ones.
    fn luminance(self) -> f64 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }

    fn channels(self) -> [u8; 3] {
        [self.r, self.g, self.b].map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8)
    }

    fn css(self) -> String {
        let [r, g, b] = self.channels();
        if self.a >= 1.0 {
            format!("#{r:02x}{g:02x}{b:02x}")
        } else {
            format!(
                "rgb({r} {g} {b} / {}%)",
                (self.a.clamp(0.0, 1.0) * 100.0).round()
            )
        }
    }

    /// The "r, g, b" form WhatsApp's `*-RGB` variables use.
    fn triplet(self) -> String {
        let [r, g, b] = self.channels();
        format!("{r}, {g}, {b}")
    }
}

/// The libadwaita colours WhatsUp reads from the theme, by their variable
/// name without "--" and "-color", in the order of `Palette::from_colors`.
pub const NAMES: [&str; 13] = [
    "window-bg",
    "window-fg",
    "view-bg",
    "view-fg",
    "headerbar-bg",
    "card-bg",
    "popover-bg",
    "accent-bg",
    "accent-fg",
    "accent",
    "destructive-bg",
    "success-bg",
    "warning-bg",
];

/// The theme's colours, as read from GTK.
#[derive(Clone, Debug, PartialEq)]
pub struct Palette {
    pub window_bg: Color,
    pub window_fg: Color,
    pub view_bg: Color,
    pub view_fg: Color,
    pub headerbar_bg: Color,
    pub card_bg: Color,
    pub popover_bg: Color,
    pub accent_bg: Color,
    pub accent_fg: Color,
    pub accent: Color,
    pub error: Color,
    pub success: Color,
    pub warning: Color,
}

impl Palette {
    /// The palette from colours in the order of `NAMES`.
    pub fn from_colors(c: [Color; 13]) -> Self {
        let [
            window_bg,
            window_fg,
            view_bg,
            view_fg,
            headerbar_bg,
            card_bg,
            popover_bg,
            accent_bg,
            accent_fg,
            accent,
            error,
            success,
            warning,
        ] = c;
        Self {
            window_bg,
            window_fg,
            view_bg,
            view_fg,
            headerbar_bg,
            card_bg,
            popover_bg,
            accent_bg,
            accent_fg,
            accent,
            error,
            success,
            warning,
        }
    }
}

/// The stylesheet that gives WhatsApp Web the palette's colours.
pub fn render(p: &Palette) -> String {
    // Colours with transparency (such as Adwaita's dark cards) are flattened
    // onto the surface they sit on, so every variable gets one solid colour.
    let window = p.window_bg.with_alpha(1.0);
    let view = p.view_bg.over(window);
    let fg = p.view_fg.over(view);
    let card = p.card_bg.over(view);
    let popover = p.popover_bg.over(window);
    let header = p.headerbar_bg.over(window);
    let accent = p.accent_bg.over(view);
    let dark = view.luminance() < 0.5;

    let tint = |c: Color, t: f64| view.mix(c.over(view), t);
    let faint = |a: f64| p.window_fg.with_alpha(a);
    let muted = view.mix(fg, 0.6);
    let outgoing = card.mix(accent, if dark { 0.35 } else { 0.18 });

    let vars: Vec<(&str, Color)> = vec![
        ("background-wash-plain", window),
        ("background-wash-inset", window),
        ("background-elevated-wash-plain", popover),
        ("background-elevated-wash-inset", popover),
        ("surface-default", view),
        ("surface-emphasized", header),
        ("components-surface-nav-bar", header),
        ("surface-elevated-default", popover),
        ("surface-elevated-emphasized", popover.mix(fg, 0.05)),
        ("surface-highlight", faint(0.07)),
        ("surface-pressed", faint(0.12)),
        ("surface-inverse", fg),
        ("components-active-list-row", faint(0.10)),
        ("components-outline-profile-photo", faint(0.10)),
        ("content-default", fg),
        ("content-action-default", fg),
        ("content-deemphasized", muted),
        ("content-disabled", view.mix(fg, 0.38)),
        ("content-inverse", view),
        ("content-on-accent", p.accent_fg.with_alpha(1.0)),
        ("content-action-emphasized", p.accent.over(view)),
        ("content-external-link", p.accent.over(view)),
        ("content-read", p.accent.over(view)),
        ("systems-status-seen", muted),
        ("lines-divider", faint(0.10)),
        ("lines-outline-deemphasized", faint(0.15)),
        ("lines-outline-default", window.mix(p.window_fg, 0.4)),
        ("accent", accent),
        ("accent-emphasized", p.accent.over(view)),
        ("accent-deemphasized", tint(p.accent_bg, 0.2)),
        ("components-filter-surface-selected", tint(p.accent_bg, 0.2)),
        ("persistent-always-branded", accent),
        ("persistent-activity-indicator", accent),
        ("systems-chat-background-wallpaper", window),
        ("systems-chat-foreground-wallpaper", faint(0.05)),
        ("systems-chat-surface-composer", card),
        ("systems-chat-surface-tray", header),
        ("systems-bubble-surface-incoming", card),
        ("systems-bubble-surface-outgoing", outgoing),
        ("systems-bubble-surface-system", card),
        ("systems-bubble-surface-business", card),
        ("systems-bubble-surface-e2e", card),
        ("systems-bubble-content-deemphasized", muted),
        ("systems-bubble-content-business", p.accent.over(view)),
        ("systems-bubble-content-e2e", p.warning.over(view)),
        ("secondary-negative", p.error.over(view)),
        ("secondary-negative-emphasized", p.error.over(view)),
        ("secondary-negative-deemphasized", tint(p.error, 0.2)),
        ("secondary-positive", p.success.over(view)),
        ("secondary-positive-deemphasized", tint(p.success, 0.2)),
        ("secondary-warning", p.warning.over(view)),
        ("secondary-warning-deemphasized", tint(p.warning, 0.2)),
    ];

    // WhatsApp sets its variables on <html> with generated class names, and
    // older rules on <body>; ours are !important on both, so they win.
    let mut css = String::from(":root,\nbody {\n");
    for (name, color) in &vars {
        css.push_str(&format!("  --WDS-{name}: {} !important;\n", color.css()));
        css.push_str(&format!(
            "  --WDS-{name}-RGB: {} !important;\n",
            color.triplet()
        ));
    }
    css.push_str(&format!(
        "  --panel-background: {} !important;\n",
        window.css()
    ));
    css.push_str("}\n");
    css
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(h: u32) -> Color {
        let c = |shift: u32| f64::from((h >> shift) & 0xff) / 255.0;
        Color::rgb(c(16), c(8), c(0))
    }

    fn adwaita_light() -> Palette {
        Palette {
            window_bg: hex(0xfafafb),
            window_fg: hex(0x000006).with_alpha(0.8),
            view_bg: hex(0xffffff),
            view_fg: hex(0x000006).with_alpha(0.8),
            headerbar_bg: hex(0xffffff),
            card_bg: hex(0xffffff),
            popover_bg: hex(0xffffff),
            accent_bg: hex(0x3584e4),
            accent_fg: hex(0xffffff),
            accent: hex(0x0461be),
            error: hex(0xe01b24),
            success: hex(0x2ec27e),
            warning: hex(0xe5a50a),
        }
    }

    #[test]
    fn colours_print_as_css() {
        assert_eq!(hex(0x0860f2).css(), "#0860f2");
        assert_eq!(hex(0x0860f2).triplet(), "8, 96, 242");
        assert_eq!(Color::WHITE.with_alpha(0.1).css(), "rgb(255 255 255 / 10%)");
    }

    #[test]
    fn transparency_is_flattened() {
        let card = Color::WHITE.with_alpha(0.08).over(hex(0x000000));
        assert_eq!(card.css(), "#141414");
        assert_eq!(hex(0x000000).mix(Color::WHITE, 0.5).css(), "#808080");
    }

    #[test]
    fn maps_the_palette_onto_whatsapp() {
        let css = render(&adwaita_light());
        assert!(css.starts_with(":root,\nbody {\n"));
        assert!(css.contains("  --WDS-accent: #3584e4 !important;\n"));
        assert!(css.contains("  --WDS-accent-RGB: 53, 132, 228 !important;\n"));
        assert!(css.contains("  --WDS-content-external-link: #0461be !important;\n"));
        assert!(css.contains("  --WDS-surface-default: #ffffff !important;\n"));
        assert!(css.contains("  --WDS-background-wash-plain: #fafafb !important;\n"));
        // Text in the theme's own (slightly transparent) colour, made solid.
        assert!(css.contains("  --WDS-content-default: #333338 !important;\n"));
        assert!(!css.contains("NaN"));
    }

    #[test]
    fn dark_themes_tint_outgoing_bubbles_more() {
        let light = render(&adwaita_light());
        let mut dark = adwaita_light();
        dark.view_bg = hex(0x1d1d20);
        dark.window_bg = hex(0x222226);
        dark.card_bg = Color::WHITE.with_alpha(0.08);
        let dark = render(&dark);
        let outgoing = |css: &str| {
            css.lines()
                .find(|l| l.contains("--WDS-systems-bubble-surface-outgoing:"))
                .unwrap()
                .to_owned()
        };
        assert_ne!(outgoing(&light), outgoing(&dark));
    }
}
