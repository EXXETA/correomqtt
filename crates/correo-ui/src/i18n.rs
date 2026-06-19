use correo_core::{
    ConnectionSettingsTab, ConnectionState, PluginLoadState, PluginSource, PluginStatus,
    PluginSurfaceTab, SettingsSection, ThemeMode, Workspace,
};
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use unic_langid::LanguageIdentifier;

const EN_US: &str = include_str!("../i18n/en-US.ftl");
const DE_DE: &str = include_str!("../i18n/de-DE.ftl");
const ES_ES: &str = include_str!("../i18n/es-ES.ftl");
const SR_LATN_RS: &str = include_str!("../i18n/sr-Latn-RS.ftl");
const FR_FR: &str = include_str!("../i18n/fr-FR.ftl");
const IT_IT: &str = include_str!("../i18n/it-IT.ftl");
const SK_SK: &str = include_str!("../i18n/sk-SK.ftl");
const KA_GE: &str = include_str!("../i18n/ka-GE.ftl");
const TLH: &str = include_str!("../i18n/tlh.ftl");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Locale {
    EnUs,
    DeDe,
    EsEs,
    SrLatnRs,
    FrFr,
    ItIt,
    SkSk,
    KaGe,
    Tlh,
}

impl Locale {
    fn from_setting(value: &str) -> Self {
        let normalized = value.trim().replace('_', "-").to_ascii_lowercase();
        if normalized == "system" {
            return system_locale();
        }
        if normalized == "de" || normalized.starts_with("de-") {
            Self::DeDe
        } else if normalized == "es" || normalized.starts_with("es-") {
            Self::EsEs
        } else if normalized == "sr" || normalized.starts_with("sr-") {
            Self::SrLatnRs
        } else if normalized == "fr" || normalized.starts_with("fr-") {
            Self::FrFr
        } else if normalized == "it" || normalized.starts_with("it-") {
            Self::ItIt
        } else if normalized == "sk" || normalized.starts_with("sk-") {
            Self::SkSk
        } else if normalized == "ka" || normalized.starts_with("ka-") {
            Self::KaGe
        } else if normalized == "tlh" {
            Self::Tlh
        } else {
            Self::EnUs
        }
    }

    fn langid(self) -> LanguageIdentifier {
        match self {
            Self::EnUs => "en-US",
            Self::DeDe => "de-DE",
            Self::EsEs => "es-ES",
            Self::SrLatnRs => "sr-Latn-RS",
            Self::FrFr => "fr-FR",
            Self::ItIt => "it-IT",
            Self::SkSk => "sk-SK",
            Self::KaGe => "ka-GE",
            Self::Tlh => "tlh",
        }
        .parse()
        .expect("bundled locale id should parse")
    }

    fn source(self) -> &'static str {
        match self {
            Self::EnUs => EN_US,
            Self::DeDe => DE_DE,
            Self::EsEs => ES_ES,
            Self::SrLatnRs => SR_LATN_RS,
            Self::FrFr => FR_FR,
            Self::ItIt => IT_IT,
            Self::SkSk => SK_SK,
            Self::KaGe => KA_GE,
            Self::Tlh => TLH,
        }
    }
}

pub(crate) struct I18n {
    locale: Locale,
    bundle: FluentBundle<FluentResource>,
}

impl I18n {
    pub(crate) fn new(language: &str) -> Self {
        let locale = Locale::from_setting(language);
        let resource = FluentResource::try_new(locale.source().to_owned())
            .expect("bundled Fluent catalog should parse");
        let mut bundle = FluentBundle::new(vec![locale.langid()]);
        bundle
            .add_resource(resource)
            .expect("bundled Fluent catalog should add cleanly");
        Self { locale, bundle }
    }

    pub(crate) fn set_language(&mut self, language: &str) {
        let locale = Locale::from_setting(language);
        if self.locale != locale {
            *self = Self::new(language);
        }
    }

    pub(crate) fn text(&self, key: &str) -> String {
        let Some(message) = self.bundle.get_message(key) else {
            return key.to_owned();
        };
        let Some(pattern) = message.value() else {
            return key.to_owned();
        };
        let mut errors = Vec::new();
        self.bundle
            .format_pattern(pattern, None, &mut errors)
            .into_owned()
    }

    pub(crate) fn text_with_args(&self, key: &str, args: &[(&str, String)]) -> String {
        let Some(message) = self.bundle.get_message(key) else {
            return key.to_owned();
        };
        let Some(pattern) = message.value() else {
            return key.to_owned();
        };
        let mut fluent_args = FluentArgs::new();
        for (name, value) in args {
            fluent_args.set(*name, value.as_str());
        }
        let mut errors = Vec::new();
        self.bundle
            .format_pattern(pattern, Some(&fluent_args), &mut errors)
            .into_owned()
    }

    pub(crate) fn product_name(&self) -> &'static str {
        match self.locale {
            Locale::Tlh => "Kore'M'kaaT",
            _ => "CorreoMQTT",
        }
    }

    pub(crate) fn workspace_label(&self, workspace: Workspace) -> String {
        self.text(match workspace {
            Workspace::Connections => "workspace-connections",
            Workspace::ImportExport => "workspace-import-export",
            Workspace::Scripts => "workspace-scripts",
            Workspace::Plugins => "workspace-plugins",
            Workspace::Diagnostics => "workspace-diagnostics",
            Workspace::Settings => "workspace-settings",
            Workspace::About => "workspace-about",
        })
    }

    pub(crate) fn theme_label(&self, mode: &ThemeMode) -> String {
        self.text(if matches!(mode, ThemeMode::System) {
            "theme-system"
        } else if mode.is_light() {
            "theme-light"
        } else {
            "theme-dark"
        })
    }

    pub(crate) fn settings_section_label(&self, section: SettingsSection) -> String {
        self.text(match section {
            SettingsSection::Appearance => "settings-appearance",
            SettingsSection::Language => "settings-language",
            SettingsSection::Search => "settings-search",
            SettingsSection::Keyring => "settings-keyring",
            SettingsSection::Updates => "settings-updates",
            SettingsSection::Plugins => "settings-plugins",
            SettingsSection::Data => "settings-data",
        })
    }

    pub(crate) fn connection_state_label(&self, state: ConnectionState) -> String {
        self.text(match state {
            ConnectionState::Disconnected => "state-disconnected",
            ConnectionState::Connecting => "state-connecting",
            ConnectionState::Connected => "state-connected",
            ConnectionState::Reconnecting => "state-reconnecting",
            ConnectionState::Error => "state-error",
        })
    }

    pub(crate) fn connection_settings_tab_label(&self, tab: ConnectionSettingsTab) -> String {
        self.text(match tab {
            ConnectionSettingsTab::Mqtt => "connection-tab-mqtt",
            ConnectionSettingsTab::Tls => "connection-tab-tls",
            ConnectionSettingsTab::Proxy => "connection-tab-proxy",
            ConnectionSettingsTab::Lwt => "connection-tab-lwt",
        })
    }

    pub(crate) fn plugin_tab_label(&self, tab: PluginSurfaceTab) -> String {
        self.text(match tab {
            PluginSurfaceTab::Installed => "plugin-tab-installed",
            PluginSurfaceTab::Marketplace => "plugin-tab-marketplace",
            PluginSurfaceTab::Configuration => "plugin-tab-configuration",
            PluginSurfaceTab::Hooks => "plugin-tab-hooks",
            PluginSurfaceTab::Diagnostics => "plugin-tab-diagnostics",
        })
    }

    pub(crate) fn plugin_load_message(&self, state: PluginLoadState) -> String {
        match state {
            PluginLoadState::Loading => self.text("plugin-load-loading"),
            PluginLoadState::Empty => self.text("plugin-load-empty"),
            PluginLoadState::Ready => String::new(),
        }
    }

    pub(crate) fn plugin_source_label(&self, source: PluginSource) -> String {
        self.text(match source {
            PluginSource::Bundled => "plugin-source-bundled",
            PluginSource::UserManifest => "plugin-source-user-manifest",
            PluginSource::LegacyJava => "plugin-source-legacy-java",
        })
    }

    pub(crate) fn plugin_status_label(&self, status: PluginStatus) -> String {
        self.text(match status {
            PluginStatus::Active => "plugin-status-active",
            PluginStatus::Disabled => "plugin-status-disabled",
            PluginStatus::NeedsConfig => "plugin-status-needs-config",
            PluginStatus::CapabilityDenied => "plugin-status-capability-denied",
            PluginStatus::LoadError => "plugin-status-load-error",
            PluginStatus::HookFailed => "plugin-status-hook-failed",
            PluginStatus::UnsupportedLegacy => "plugin-status-unsupported-legacy",
        })
    }

    pub(crate) fn language_option_label(&self, id: &str, fallback: &str) -> String {
        match id {
            "system" => self.text("common-system"),
            _ => origin_language_name(id).unwrap_or(fallback).to_owned(),
        }
    }

    pub(crate) fn language_menu_option_label(
        &self,
        id: &str,
        fallback: &str,
        current: &str,
    ) -> String {
        if id == "system" || same_language_option(id, current) {
            return self.language_option_label(id, fallback);
        }

        let Some(origin) = origin_language_name(id) else {
            return fallback.to_owned();
        };
        let Some(key) = language_label_key(id) else {
            return origin.to_owned();
        };
        let translated = self.text(key);
        format!("{translated} ({origin})")
    }
}

fn origin_language_name(id: &str) -> Option<&'static str> {
    match id {
        "en_US" | "en-US" => Some("English"),
        "de_DE" | "de-DE" => Some("Deutsch"),
        "es_ES" | "es-ES" => Some("Español"),
        "sr_RS" | "sr-Latn-RS" | "sr_RS_Latn" => Some("Srpski"),
        "fr_FR" | "fr-FR" => Some("Français"),
        "it_IT" | "it-IT" => Some("Italiano"),
        "sk_SK" | "sk-SK" => Some("Slovenčina"),
        "ka_GE" | "ka-GE" => Some("ქართული"),
        "tlh" => Some("tlhIngan Hol"),
        _ => None,
    }
}

fn language_label_key(id: &str) -> Option<&'static str> {
    match id {
        "en_US" | "en-US" => Some("language-english"),
        "de_DE" | "de-DE" => Some("language-german"),
        "es_ES" | "es-ES" => Some("language-spanish"),
        "sr_RS" | "sr-Latn-RS" | "sr_RS_Latn" => Some("language-serbian"),
        "fr_FR" | "fr-FR" => Some("language-french"),
        "it_IT" | "it-IT" => Some("language-italian"),
        "sk_SK" | "sk-SK" => Some("language-slovak"),
        "ka_GE" | "ka-GE" => Some("language-georgian"),
        "tlh" => Some("language-klingon"),
        _ => None,
    }
}

fn same_language_option(left: &str, right: &str) -> bool {
    language_label_key(left).is_some() && language_label_key(left) == language_label_key(right)
}

pub(crate) fn language_option_visible(id: &str, current: &str, klingon_unlocked: bool) -> bool {
    id != "tlh" || klingon_unlocked || current == "tlh"
}

fn system_locale() -> Locale {
    std::env::var("LANG")
        .ok()
        .as_deref()
        .map(Locale::from_setting)
        .unwrap_or(Locale::EnUs)
}

#[cfg(test)]
mod tests {
    use super::I18n;
    use correo_core::{ThemeMode, Workspace};

    #[test]
    fn german_catalog_uses_original_settings_label() {
        let i18n = I18n::new("de_DE");

        assert_eq!(i18n.text("settings-header"), "Einstellungen");
        assert_eq!(i18n.text("common-save"), "Speichern");
        assert_eq!(i18n.text("plugin-header"), "Plugins");
        assert_eq!(i18n.workspace_label(Workspace::Settings), "Einstellungen");
        assert_eq!(i18n.theme_label(&ThemeMode::Dark), "Dunkel");
    }

    #[test]
    fn unsupported_locale_falls_back_to_english() {
        let i18n = I18n::new("pt_BR");

        assert_eq!(i18n.text("settings-header"), "Settings");
        assert_eq!(i18n.workspace_label(Workspace::Connections), "Connections");
    }

    #[test]
    fn spanish_and_klingon_catalogs_are_available() {
        let spanish = I18n::new("es_ES");
        let klingon = I18n::new("tlh");

        assert_eq!(spanish.text("settings-header"), "Configuración");
        assert_eq!(klingon.text("settings-language"), "Hol wIv");
    }

    #[test]
    fn language_menu_labels_add_origin_for_non_current_languages() {
        let i18n = I18n::new("de_DE");

        assert_eq!(
            i18n.language_menu_option_label("en_US", "", "de_DE"),
            "Englisch (English)"
        );
        assert_eq!(
            i18n.language_menu_option_label("de_DE", "", "de_DE"),
            "Deutsch"
        );
        assert_eq!(
            i18n.language_menu_option_label("fr_FR", "", "de_DE"),
            "Französisch (Français)"
        );
        assert_eq!(
            i18n.language_menu_option_label("ka_GE", "", "de_DE"),
            "Georgisch (ქართული)"
        );
        assert_eq!(
            i18n.language_menu_option_label("tlh", "", "de_DE"),
            "Klingonisch (tlhIngan Hol)"
        );
    }

    #[test]
    fn georgian_language_menu_labels_include_translated_names() {
        let i18n = I18n::new("ka_GE");

        assert_eq!(
            i18n.language_menu_option_label("en_US", "", "ka_GE"),
            "ინგლისური (English)"
        );
        assert_eq!(
            i18n.language_menu_option_label("ka_GE", "", "ka_GE"),
            "ქართული"
        );
    }

    #[test]
    fn additional_catalogs_are_available() {
        assert_eq!(I18n::new("sr_RS").text("settings-header"), "Podešavanja");
        assert_eq!(I18n::new("fr_FR").text("settings-header"), "Paramètres");
        assert_eq!(I18n::new("it_IT").text("settings-header"), "Impostazioni");
        assert_eq!(I18n::new("sk_SK").text("settings-header"), "Nastavenia");
        assert_eq!(I18n::new("ka_GE").text("settings-header"), "პარამეტრები");
    }
}
