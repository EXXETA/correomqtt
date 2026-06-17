mod topics;
use topics::metadata_for_topic;

pub const ACTION_ID: &str = "system-topics";
pub const TOPIC_FILTER: &str = "$SYS/#";

pub const PLUGIN_ID: &str = "org.correomqtt.plugins.system-topic";
pub const LEGACY_PLUGIN_ID: &str = "systopic";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SysTopicHostAction {
    Subscribe { topic_filter: String },
    Unsubscribe { topic_filter: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SysTopicOpenWindow {
    pub title: String,
    pub message_filter_prefix: String,
    pub latest_per_topic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SysTopicActionResponse {
    pub host_actions: Vec<SysTopicHostAction>,
    pub open_window: SysTopicOpenWindow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SysTopicCloseResponse {
    pub host_actions: Vec<SysTopicHostAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SysTopicMessage {
    pub topic: String,
    pub payload: Vec<u8>,
    pub qos: String,
    pub retained: bool,
    pub timestamp: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SysTopicUiNode {
    Heading {
        text: String,
    },
    Label {
        text: String,
    },
    Separator,
    Table {
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    MetricList(SysTopicMetricList),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SysTopicMetricList {
    pub broker: String,
    pub latest_update: String,
    pub copy_text: String,
    pub rows: Vec<SysTopicMetricRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SysTopicMetricRow {
    pub name: String,
    pub description: String,
    pub value: String,
}

pub fn connection_action_clicked(
    action_id: &str,
    connection_name: &str,
) -> Option<SysTopicActionResponse> {
    if action_id != ACTION_ID {
        return None;
    }
    Some(SysTopicActionResponse {
        host_actions: vec![SysTopicHostAction::Subscribe {
            topic_filter: TOPIC_FILTER.to_owned(),
        }],
        open_window: SysTopicOpenWindow {
            title: format!("SysTopics for {connection_name}"),
            message_filter_prefix: "$SYS/".to_owned(),
            latest_per_topic: true,
        },
    })
}

pub fn connection_window_closed(action_id: &str) -> Option<SysTopicCloseResponse> {
    if action_id != ACTION_ID {
        return None;
    }
    Some(SysTopicCloseResponse {
        host_actions: vec![SysTopicHostAction::Unsubscribe {
            topic_filter: TOPIC_FILTER.to_owned(),
        }],
    })
}

pub fn render_window(
    action_id: &str,
    broker: &str,
    messages: &[SysTopicMessage],
) -> Option<Vec<SysTopicUiNode>> {
    if action_id != ACTION_ID {
        return None;
    }

    let mut latest = std::collections::BTreeMap::<String, &SysTopicMessage>::new();
    for message in messages {
        if message.topic.starts_with("$SYS/") {
            latest.entry(message.topic.clone()).or_insert(message);
        }
    }

    let mut rows = Vec::new();
    let mut latest_update = String::new();
    for (topic, message) in latest {
        let metadata = metadata_for_topic(&topic);
        let label = metadata
            .map(|item| item.label)
            .unwrap_or("System topic")
            .to_owned();
        let description = metadata
            .map(|item| item.description)
            .unwrap_or("Unrecognized $SYS broker metric.")
            .to_owned();
        if message.timestamp > latest_update {
            latest_update = message.timestamp.clone();
        }
        rows.push(SysTopicMetricRow {
            name: label,
            description,
            value: String::from_utf8_lossy(&message.payload).into_owned(),
        });
    }

    let mut nodes = Vec::new();
    if rows.is_empty() {
        nodes.push(SysTopicUiNode::Label {
            text: "No $SYS messages received yet.".to_owned(),
        });
    } else {
        let copy_text = rows
            .iter()
            .map(|row| format!("{}\t{}", row.name, row.value))
            .collect::<Vec<_>>()
            .join("\n");
        nodes.push(SysTopicUiNode::MetricList(SysTopicMetricList {
            broker: broker.to_owned(),
            latest_update,
            copy_text,
            rows,
        }));
    }
    Some(nodes)
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn correomqtt_alloc(len: i32) -> i32 {
    if len <= 0 {
        return 0;
    }

    let Ok(layout) = std::alloc::Layout::from_size_align(len as usize, 1) else {
        return 0;
    };
    unsafe { std::alloc::alloc(layout) as i32 }
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn correomqtt_dealloc(ptr: i32, len: i32) {
    if ptr <= 0 || len <= 0 {
        return;
    }

    if let Ok(layout) = std::alloc::Layout::from_size_align(len as usize, 1) {
        unsafe {
            std::alloc::dealloc(ptr as *mut u8, layout);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_click_subscribes_and_opens_window() {
        let response = connection_action_clicked(ACTION_ID, "Local Broker").unwrap();

        assert_eq!(response.open_window.title, "SysTopics for Local Broker");
        assert_eq!(
            response.host_actions,
            vec![SysTopicHostAction::Subscribe {
                topic_filter: TOPIC_FILTER.to_owned()
            }]
        );
    }

    #[test]
    fn window_close_unsubscribes() {
        let response = connection_window_closed(ACTION_ID).unwrap();

        assert_eq!(
            response.host_actions,
            vec![SysTopicHostAction::Unsubscribe {
                topic_filter: TOPIC_FILTER.to_owned()
            }]
        );
    }

    #[test]
    fn render_window_shows_latest_known_topic() {
        let nodes = render_window(
            ACTION_ID,
            "localhost:1883",
            &[
                SysTopicMessage {
                    topic: "$SYS/broker/clients/connected".to_owned(),
                    payload: b"7".to_vec(),
                    qos: "QoS 0".to_owned(),
                    retained: false,
                    timestamp: "10:00".to_owned(),
                },
                SysTopicMessage {
                    topic: "devices/demo".to_owned(),
                    payload: b"ignored".to_vec(),
                    qos: "QoS 0".to_owned(),
                    retained: false,
                    timestamp: "10:01".to_owned(),
                },
            ],
        )
        .unwrap();

        let table = nodes
            .iter()
            .find_map(|node| match node {
                SysTopicUiNode::MetricList(list) => Some(&list.rows),
                _ => None,
            })
            .unwrap();
        assert_eq!(table.len(), 1);
        assert_eq!(table[0].name, "Connected clients");
        assert_eq!(table[0].value, "7");
    }
}
