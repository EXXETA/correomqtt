#[cfg(feature = "egui")]
use egui::{vec2, Margin, Vec2};

pub const CONTROL_PADDING: i8 = 8;
pub const BUTTON_HORIZONTAL_PADDING: f32 = 16.0;
pub const CONTROL_HEIGHT: f32 = 34.0;
pub const FONT_SIZE_SCALE: f32 = 1.15;
pub const CORNER_RADIUS: u8 = 4;

pub const HEADER_HEIGHT: f32 = 64.0;
pub const HEADER_MARGIN_X: i8 = 16;
pub const HEADER_MARGIN_Y: i8 = 6;
pub const HEADER_ICON_SIZE: f32 = 44.0;
pub const APP_TITLE_BASE_SIZE: f32 = 16.0;
pub const APP_TITLE_SIZE: f32 = APP_TITLE_BASE_SIZE * 1.75;

pub const RAIL_WIDTH: f32 = 48.0;
pub const RAIL_MARGIN: i8 = 4;
pub const MIN_WINDOW_WIDTH: f32 = 640.0;
pub const NARROW_SIDEBAR_WINDOW_WIDTH: f32 = 1120.0;
pub const SIDEBAR_DEFAULT_WIDTH: f32 = 345.0;
pub const SIDEBAR_MIN_WIDTH: f32 = 345.0;
pub const SIDEBAR_MAX_WIDTH: f32 = 600.0;
pub const CONNECTION_CONTEXT_FLYOUT_SCREEN_WIDTH: f32 = 900.0;
pub const CONNECTION_FLYOUT_WIDTH: f32 = 345.0;
pub const CONNECTION_SIDEBAR_MIN_WIDTH: f32 = CONNECTION_FLYOUT_WIDTH;
pub const CONNECTION_SIDEBAR_MAX_WIDTH: f32 = 600.0;
pub const SCRIPTING_CONTEXT_FLYOUT_SCREEN_WIDTH: f32 = 900.0;
pub const SCRIPTING_FLYOUT_WIDTH: f32 = CONNECTION_FLYOUT_WIDTH;
pub const PLUGIN_CONTEXT_FLYOUT_SCREEN_WIDTH: f32 = 900.0;
pub const PLUGIN_FLYOUT_WIDTH: f32 = CONNECTION_FLYOUT_WIDTH;
pub const RECOVERY_CONTEXT_DEFAULT_WIDTH: f32 = 280.0;
pub const RECOVERY_CONTEXT_MIN_WIDTH: f32 = 240.0;
pub const RECOVERY_CONTEXT_MAX_WIDTH: f32 = 360.0;
pub const SIDEBAR_MARGIN_LEFT: i8 = 21;
pub const SIDEBAR_MARGIN_RIGHT: i8 = SIDEBAR_MARGIN_LEFT;
pub const SIDEBAR_MARGIN_TOP: i8 = 12;
pub const SIDEBAR_MARGIN_BOTTOM: i8 = 10;
pub const CENTRAL_MARGIN: i8 = 12;

pub const SETTINGS_LABEL_WIDTH: f32 = 220.0;
pub const SETTINGS_CONTROL_WIDTH: f32 = 420.0;
pub const SETTINGS_COMBO_WIDTH: f32 = 180.0;
pub const HEADER_THEME_SELECTOR_WIDTH: f32 = 96.0;
pub const HEADER_LANGUAGE_SELECTOR_WIDTH: f32 = 124.0;

pub const TOOLBAR_GAP: f32 = 8.0;
pub const TABLE_HEADER_HEIGHT: f32 = 22.0;
pub const TABLE_ROW_HEIGHT: f32 = 30.0;
pub const TABLE_MIN_HEIGHT: f32 = 96.0;
pub const TABLE_SCROLL_BOTTOM_GAP: f32 = TILE_PADDING_X as f32;
pub const PUBLISH_ACTION_BUTTON_WIDTH: f32 = 104.0;
pub const SUBSCRIBE_ACTION_BUTTON_WIDTH: f32 = 116.0;
pub const QOS_WIDTH: f32 = 96.0;
pub const RETAINED_WIDTH: f32 = 142.0;
pub const SUBSCRIPTION_ROW_HEIGHT: f32 = 40.0;
pub const SUBSCRIPTION_ROW_PADDING_X: f32 = 10.0;
pub const SUBSCRIPTION_ROW_PADDING_RIGHT: f32 = 4.0;
pub const SUBSCRIPTION_TOGGLE_WIDTH: f32 = 56.0;
pub const SUBSCRIPTION_QOS_SLOT_WIDTH: f32 = 58.0;
pub const SUBSCRIPTION_QOS_PILL_WIDTH: f32 = 54.0;
pub const SUBSCRIPTION_QOS_PILL_HEIGHT: f32 = 24.0;
pub const PUBLISH_HISTORY_TIME_WIDTH: f32 = 72.0;
pub const PUBLISH_HISTORY_QOS_WIDTH: f32 = 60.0;
pub const PUBLISH_HISTORY_BYTES_WIDTH: f32 = 58.0;
pub const MESSAGE_TABLE_TIME_WIDTH: f32 = 66.0;
pub const MESSAGE_TABLE_QOS_WIDTH: f32 = 52.0;
pub const MESSAGE_TABLE_BYTES_WIDTH: f32 = 58.0;
pub const MESSAGE_TABLE_BADGES_WIDTH: f32 = 66.0;
pub const MESSAGE_ROW_META_WIDTH: f32 = 190.0;
pub const MESSAGE_ROW_TOPIC_META_GAP: f32 = 30.0;
pub const MESSAGE_ROW_PADDING_RIGHT: f32 = 18.0;
pub const MESSAGE_TABLE_ROW_HEIGHT: f32 = 52.0;

pub const WORKBENCH_DEFAULT_CENTER_RATIO: f32 = 0.5;
pub const WORKBENCH_DEFAULT_STACK_RATIO: f32 = 0.55;
pub const WORKBENCH_DIVIDER_SIZE: f32 = 8.0;
pub const WORKBENCH_MIN_PANE_WIDTH: f32 = 345.0;
pub const WORKBENCH_MIN_TOP_HEIGHT: f32 = 180.0;
pub const WORKBENCH_MIN_BOTTOM_HEIGHT: f32 = 150.0;
pub const WORKBENCH_NARROW_WIDTH: f32 = 760.0;
pub const WORKBENCH_ICON_ONLY_ACTION_WIDTH: f32 = 520.0;
pub const WORKBENCH_PANE_PADDING_X: f32 = 0.0;
pub const WORKBENCH_PANE_PADDING_Y: f32 = 12.0;
pub const WORKBENCH_CENTER_SPLIT_GUTTER: f32 = 12.0;
pub const WORKBENCH_MODE_BUTTON_GAP: f32 = 2.0;

pub const TILE_GAP: f32 = 0.0;
pub const TILE_LINE_GAP: f32 = 0.0;
pub const TWO_LINE_TILE_HEIGHT: f32 = 50.0;
pub const TILE_PADDING_X: i8 = 12;
pub const TILE_PADDING_TOP: i8 = 8;
pub const TILE_PADDING_BOTTOM: i8 = 8;
pub const TILE_SCROLLBAR_INSET: f32 = 4.0;

pub const CHECKBOX_ICON_SCALE: f32 = 1.6;
pub const CHECKBOX_TEXT_TRAILING_PADDING: f32 = 8.0;
pub const TEXT_EDIT_FOCUS_BAR_WIDTH: f32 = 3.0;

#[cfg(feature = "egui")]
pub fn control_margin() -> Margin {
    Margin::same(CONTROL_PADDING)
}

#[cfg(feature = "egui")]
pub fn control_padding() -> Vec2 {
    vec2(CONTROL_PADDING as f32, CONTROL_PADDING as f32)
}

#[cfg(feature = "egui")]
pub fn button_padding() -> Vec2 {
    vec2(BUTTON_HORIZONTAL_PADDING, CONTROL_PADDING as f32)
}

#[cfg(feature = "egui")]
pub fn square_icon_button_side() -> f32 {
    CONTROL_HEIGHT
}

#[cfg(feature = "egui")]
pub fn square_icon_button_size() -> [f32; 2] {
    [square_icon_button_side(), square_icon_button_side()]
}

#[cfg(feature = "egui")]
pub fn sidebar_margin() -> Margin {
    Margin {
        left: SIDEBAR_MARGIN_LEFT,
        right: SIDEBAR_MARGIN_RIGHT,
        top: SIDEBAR_MARGIN_TOP,
        bottom: SIDEBAR_MARGIN_BOTTOM,
    }
}

#[cfg(feature = "egui")]
pub fn tile_inner_margin() -> Margin {
    Margin {
        left: TILE_PADDING_X,
        right: TILE_PADDING_X,
        top: TILE_PADDING_TOP,
        bottom: TILE_PADDING_BOTTOM,
    }
}

#[cfg(feature = "egui")]
pub fn tile_inner_padding() -> Vec2 {
    vec2(TILE_PADDING_X as f32, TILE_PADDING_TOP as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_control_metrics_are_unified() {
        assert_eq!(CONTROL_PADDING, 8);
        assert_eq!(BUTTON_HORIZONTAL_PADDING, 16.0);
        assert_eq!(CONTROL_HEIGHT, 34.0);
    }

    #[test]
    fn header_metrics_match_application_chrome() {
        assert_eq!(HEADER_HEIGHT, 64.0);
        assert_eq!(APP_TITLE_SIZE, APP_TITLE_BASE_SIZE * 1.75);
    }
}
