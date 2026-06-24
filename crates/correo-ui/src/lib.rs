mod about;
mod broker;
mod command_bar;
mod connection_launcher;
mod connection_plugins;
mod connection_settings;
mod diagnostics;
mod i18n;
mod icons;
mod migration_recovery;
mod modal_style;
mod motion;
mod nav;
mod overlay_bounds;
mod payload_highlight;
mod plugins;
mod responsive;
mod scripts;
mod settings;
mod shell;
mod time_format;
mod toasts;
mod transfer_wizard;
mod transfer_wizard_rows;
mod widgets;
mod workbench;
mod workbench_connection_messages;
mod workbench_connection_messages_filters;
mod workbench_connection_messages_text;
mod workbench_detail;
mod workbench_dialogs;
mod workbench_header;
mod workbench_helpers;
mod workbench_layout;
mod workbench_messages;
mod workbench_plugin_windows;
mod workbench_publish;
mod workbench_subscribe;
mod workspace;

pub use shell::{stored_theme, CorreoUi, THEME_KEY};

pub type PayloadHighlighter = std::sync::Arc<
    dyn Fn(&str, &[String]) -> Option<Vec<correo_core::PayloadSyntaxSpan>> + Send + Sync,
>;

pub mod theme {
    pub use correo_style::layout::{
        button_padding, control_margin, control_padding, CONTROL_HEIGHT, CONTROL_PADDING,
    };
    pub use correo_style::*;
}
