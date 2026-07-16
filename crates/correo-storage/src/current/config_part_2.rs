#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginHookKind {
    IncomingTransform,
    OutgoingTransform,
    Validator,
    DetailTransform,
    #[default]
    DetailFormatter,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginStateSettings {
    pub enabled: bool,
}

impl Default for PluginStateSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThemeSettings {
    pub active_theme: Option<Theme>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Theme {
    pub name: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GlobalUiSettings {
    pub window_position_x: f64,
    pub window_position_y: f64,
    pub window_width: f64,
    pub window_height: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ConnectionUiSettings {
    pub show_subscribe: bool,
    pub show_publish: bool,
    pub main_divider_position: f64,
    pub publish_divider_position: f64,
    pub publish_detail_divider_position: f64,
    pub publish_detail_active: bool,
    pub subscribe_divider_position: f64,
    pub subscribe_detail_divider_position: f64,
    pub subscribe_detail_active: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MessageListViewConfig {
    pub label_visibility: BTreeMap<LabelType, bool>,
}

#[derive(Clone, Copy, Debug, Ord, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub enum LabelType {
    Qos,
    Retained,
    Timestamp,
}

/// Messaging protocol of a connection. Only MQTT ships today; the enum exists
/// so a second protocol can be added later without changing the config schema
/// shape or breaking existing serialized profiles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    #[default]
    Mqtt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MqttVersion {
    Mqtt311,
    Mqtt50,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TlsSsl {
    Off,
    Keystore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Proxy {
    Off,
    Ssh,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Auth {
    Off,
    Password,
    Keyfile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lwt {
    Off,
    On,
}
