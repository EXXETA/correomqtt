use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;

pub const LIGHT_THEME_ID: &str = "builtin/light";
pub const DARK_THEME_ID: &str = "builtin/dark";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuiltinTheme {
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ThemeId {
    Builtin(BuiltinTheme),
    Plugin(String),
}

impl ThemeId {
    pub const LIGHT: Self = Self::Builtin(BuiltinTheme::Light);
    pub const DARK: Self = Self::Builtin(BuiltinTheme::Dark);

    pub fn parse(value: impl AsRef<str>) -> Self {
        match canonical_theme_name(value.as_ref()) {
            ThemeName::System => Self::DARK,
            ThemeName::Light => Self::LIGHT,
            ThemeName::Dark => Self::DARK,
            ThemeName::Plugin(id) => Self::Plugin(id.to_owned()),
        }
    }

    pub fn as_str(&self) -> Cow<'_, str> {
        match self {
            Self::Builtin(BuiltinTheme::Light) => Cow::Borrowed(LIGHT_THEME_ID),
            Self::Builtin(BuiltinTheme::Dark) => Cow::Borrowed(DARK_THEME_ID),
            Self::Plugin(id) => Cow::Borrowed(id.as_str()),
        }
    }

    pub fn is_builtin(&self) -> bool {
        matches!(self, Self::Builtin(_))
    }
}

impl Serialize for ThemeId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str().as_ref())
    }
}

impl<'de> Deserialize<'de> for ThemeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Self::parse)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ThemeSelection {
    #[default]
    System,
    Theme(ThemeId),
}

impl ThemeSelection {
    #[allow(non_upper_case_globals)]
    pub const Light: Self = Self::Theme(ThemeId::LIGHT);
    #[allow(non_upper_case_globals)]
    pub const Dark: Self = Self::Theme(ThemeId::DARK);
    pub const ALL: [Self; 3] = [Self::System, Self::Light, Self::Dark];

    pub fn parse(value: impl AsRef<str>) -> Self {
        match canonical_theme_name(value.as_ref()) {
            ThemeName::System => Self::System,
            ThemeName::Light => Self::Light,
            ThemeName::Dark => Self::Dark,
            ThemeName::Plugin(id) => Self::Theme(ThemeId::Plugin(id.to_owned())),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Theme(ThemeId::Builtin(BuiltinTheme::Light)) => "Light",
            Self::Theme(ThemeId::Builtin(BuiltinTheme::Dark)) => "Dark",
            Self::Theme(ThemeId::Plugin(_)) => "Plugin theme",
        }
    }

    pub fn storage_name(&self) -> Cow<'_, str> {
        match self {
            Self::System => Cow::Borrowed("system"),
            Self::Theme(id) => id.as_str(),
        }
    }

    pub fn is_light(&self) -> bool {
        matches!(self, Self::Theme(ThemeId::Builtin(BuiltinTheme::Light)))
    }

    pub fn is_dark(&self) -> bool {
        matches!(self, Self::Theme(ThemeId::Builtin(BuiltinTheme::Dark)))
    }
}

impl Serialize for ThemeSelection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.storage_name().as_ref())
    }
}

impl<'de> Deserialize<'de> for ThemeSelection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Self::parse)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColorRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ColorRgb {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const fn gray(value: u8) -> Self {
        Self::rgb(value, value, value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeColors {
    pub window_bg: ColorRgb,
    pub panel_bg: ColorRgb,
    pub panel_raised: ColorRgb,
    pub field_bg: ColorRgb,
    pub border: ColorRgb,
    pub text_primary: ColorRgb,
    pub text_secondary: ColorRgb,
    pub text_disabled: ColorRgb,
    pub accent: ColorRgb,
    pub accent_selected_bg: ColorRgb,
    pub success: ColorRgb,
    pub warning: ColorRgb,
    pub danger: ColorRgb,
    pub script: ColorRgb,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeDefinition {
    pub id: ThemeId,
    pub name: String,
    pub colors: ThemeColors,
}

#[derive(Debug, Clone)]
pub struct ThemeRegistry {
    themes: Vec<ThemeDefinition>,
}

impl ThemeRegistry {
    pub fn builtins() -> Self {
        Self {
            themes: vec![light_theme(), dark_theme()],
        }
    }

    pub fn register(&mut self, theme: ThemeDefinition) {
        if let Some(existing) = self.themes.iter_mut().find(|item| item.id == theme.id) {
            *existing = theme;
        } else {
            self.themes.push(theme);
        }
    }

    pub fn resolve(&self, id: &ThemeId) -> Option<&ThemeDefinition> {
        self.themes.iter().find(|theme| theme.id == *id)
    }

    pub fn themes(&self) -> &[ThemeDefinition] {
        &self.themes
    }
}

pub fn light_theme() -> ThemeDefinition {
    ThemeDefinition {
        id: ThemeId::LIGHT,
        name: "Light".to_owned(),
        colors: ThemeColors {
            window_bg: ColorRgb::rgb(0xF0, 0xF3, 0xF7),
            panel_bg: ColorRgb::rgb(0xFF, 0xFF, 0xFF),
            panel_raised: ColorRgb::rgb(0xF8, 0xFA, 0xFC),
            field_bg: ColorRgb::rgb(0xFF, 0xFF, 0xFF),
            border: ColorRgb::rgb(0xD7, 0xDE, 0xE6),
            text_primary: ColorRgb::rgb(0x17, 0x20, 0x2A),
            text_secondary: ColorRgb::rgb(0x5F, 0x6B, 0x77),
            text_disabled: ColorRgb::rgb(0x9A, 0xA6, 0xB2),
            accent: ColorRgb::rgb(0x17, 0x6E, 0xA3),
            accent_selected_bg: ColorRgb::rgb(0xDC, 0xEE, 0xFF),
            success: ColorRgb::rgb(0x1F, 0x8E, 0x58),
            warning: ColorRgb::rgb(0xA9, 0x64, 0x00),
            danger: ColorRgb::rgb(0xB3, 0x3A, 0x3A),
            script: ColorRgb::rgb(0x4F, 0x7E, 0x23),
        },
    }
}

pub fn dark_theme() -> ThemeDefinition {
    ThemeDefinition {
        id: ThemeId::DARK,
        name: "Dark".to_owned(),
        colors: ThemeColors {
            window_bg: ColorRgb::gray(0x18),
            panel_bg: ColorRgb::gray(0x20),
            panel_raised: ColorRgb::gray(0x28),
            field_bg: ColorRgb::gray(0x08),
            border: ColorRgb::gray(0x56),
            text_primary: ColorRgb::gray(0xEE),
            text_secondary: ColorRgb::gray(0xB0),
            text_disabled: ColorRgb::gray(0x72),
            accent: ColorRgb::rgb(0x2E, 0x8F, 0xCA),
            accent_selected_bg: ColorRgb::rgb(0x1F, 0x4F, 0x6F),
            success: ColorRgb::rgb(0x3F, 0xB9, 0x74),
            warning: ColorRgb::rgb(0xE4, 0xA3, 0x43),
            danger: ColorRgb::rgb(0xD9, 0x5C, 0x5C),
            script: ColorRgb::rgb(0x8F, 0xBF, 0x4D),
        },
    }
}

enum ThemeName<'a> {
    System,
    Light,
    Dark,
    Plugin(&'a str),
}

fn canonical_theme_name(value: &str) -> ThemeName<'_> {
    match value.trim() {
        "System" | "system" => ThemeName::System,
        "Light" | "light" | "Light Legacy" | "light_legacy" | LIGHT_THEME_ID => ThemeName::Light,
        "Dark" | "dark" | "Dark Legacy" | "dark_legacy" | DARK_THEME_ID => ThemeName::Dark,
        id => ThemeName::Plugin(id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_theme_names_map_to_builtin_ids() {
        assert_eq!(ThemeSelection::parse("Light Legacy"), ThemeSelection::Light);
        assert_eq!(ThemeSelection::parse("dark_legacy"), ThemeSelection::Dark);
        assert_eq!(ThemeSelection::parse("System"), ThemeSelection::System);
    }

    #[test]
    fn plugin_theme_ids_are_preserved() {
        assert_eq!(
            ThemeSelection::parse("plugin.example/custom"),
            ThemeSelection::Theme(ThemeId::Plugin("plugin.example/custom".to_owned()))
        );
    }

    #[test]
    fn dark_neutral_colors_are_grayscale() {
        let colors = dark_theme().colors;
        for color in [
            colors.window_bg,
            colors.panel_bg,
            colors.panel_raised,
            colors.field_bg,
            colors.border,
            colors.text_primary,
            colors.text_secondary,
            colors.text_disabled,
        ] {
            assert_eq!(color.r, color.g);
            assert_eq!(color.g, color.b);
        }
    }

    #[test]
    fn dark_surfaces_have_increasing_brightness() {
        let colors = dark_theme().colors;
        assert!(colors.field_bg.r < colors.window_bg.r);
        assert!(colors.window_bg.r < colors.panel_bg.r);
        assert!(colors.panel_bg.r < colors.panel_raised.r);
    }

    #[test]
    fn registry_replaces_duplicate_theme_ids() {
        let mut registry = ThemeRegistry::builtins();
        let mut theme = light_theme();
        theme.name = "Custom light".to_owned();
        registry.register(theme);
        assert_eq!(registry.themes().len(), 2);
        assert_eq!(
            registry.resolve(&ThemeId::LIGHT).unwrap().name,
            "Custom light"
        );
    }
}
