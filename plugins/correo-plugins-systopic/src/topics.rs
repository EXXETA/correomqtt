#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SysTopicMetadata<'a> {
    pub label: &'a str,
    pub description: &'a str,
    pub window: Option<&'a str>,
}

pub fn metadata_for_topic(topic: &str) -> Option<SysTopicMetadata<'_>> {
    match topic {
        "$SYS/broker/version" => Some(SysTopicMetadata {
            label: "Version",
            description: "The version of the broker.",
            window: None,
        }),
        "$SYS/broker/uptime" => Some(SysTopicMetadata {
            label: "Uptime",
            description: "The amount of time in seconds the broker has been online.",
            window: None,
        }),
        "$SYS/broker/time" => Some(SysTopicMetadata {
            label: "Time",
            description: "The current time on the server.",
            window: None,
        }),
        "$SYS/broker/changeset" => Some(SysTopicMetadata {
            label: "Changeset",
            description: "The repository changeset (revision) associated with this build. Static.",
            window: None,
        }),
        "$SYS/broker/build_time" => Some(SysTopicMetadata {
            label: "Build time",
            description: "The timestamp at which this particular build of the broker was made.",
            window: None,
        }),
        "$SYS/broker/retained messages/count" | "$SYS/broker/retained messages" => {
            Some(SysTopicMetadata {
                label: "Retained messages",
                description: "The total number of retained messages active on the broker.",
                window: None,
            })
        }
        "$SYS/broker/subscriptions/count" => Some(SysTopicMetadata {
            label: "Subscriptions",
            description: "The total number of subscriptions active on the broker.",
            window: None,
        }),
        "$SYS/broker/clients/connected" => Some(SysTopicMetadata {
            label: "Connected clients",
            description: "The number of currently connected clients.",
            window: None,
        }),
        "$SYS/broker/clients/active" => Some(SysTopicMetadata {
            label: "Active clients",
            description: "The number of currently active clients.",
            window: None,
        }),
        "$SYS/broker/clients/disconnected" => Some(SysTopicMetadata {
            label: "Disconnected clients",
            description: "The total number of persistent clients (with clean session disabled) that are registered at the broker but are currently disconnected.",
            window: None,
        }),
        "$SYS/broker/clients/maximum" => Some(SysTopicMetadata {
            label: "Max. connected clients",
            description: "The maximum number of active clients that have been connected to the broker. This is only calculated when the $SYS topic tree is updated, so short lived client connections may not be counted.",
            window: None,
        }),
        "$SYS/broker/clients/total" => Some(SysTopicMetadata {
            label: "Total clients",
            description: "The total number of connected and disconnected clients with a persistent session currently connected and registered on the broker.",
            window: None,
        }),
        "$SYS/broker/messages/received" => Some(SysTopicMetadata {
            label: "Received messages",
            description: "The total number of messages of any type received since the broker started.",
            window: None,
        }),
        "$SYS/broker/messages/sent" => Some(SysTopicMetadata {
            label: "Sent messages",
            description: "The total number of messages of any type sent since the broker started.",
            window: None,
        }),
        "$SYS/broker/messages/dropped" => Some(SysTopicMetadata {
            label: "Dropped publishes",
            description: "The total number of publish messages that have been dropped due to inflight/queuing limits.",
            window: None,
        }),
        "$SYS/broker/publish/messages/received" => Some(SysTopicMetadata {
            label: "Received publishes",
            description: "The total number of PUBLISH messages received since the broker started.",
            window: None,
        }),
        "$SYS/broker/publish/messages/sent" => Some(SysTopicMetadata {
            label: "Sent publishes",
            description: "The total number of PUBLISH messages sent since the broker started.",
            window: None,
        }),
        "$SYS/broker/bytes/received" => Some(SysTopicMetadata {
            label: "Received bytes",
            description: "The total number of bytes received since the broker started.",
            window: None,
        }),
        "$SYS/broker/bytes/sent" => Some(SysTopicMetadata {
            label: "Sent bytes",
            description: "The total number of bytes sent since the broker started.",
            window: None,
        }),
        "$SYS/broker/publish/bytes/sent" => Some(SysTopicMetadata {
            label: "Sent publish bytes",
            description: "The total number of bytes sent with publishes since the broker started.",
            window: None,
        }),
        "$SYS/broker/messages/inflight" => Some(SysTopicMetadata {
            label: "Inflight",
            description: "The number of messages with QoS>0 that are awaiting acknowledgments.",
            window: None,
        }),
        "$SYS/broker/messages/stored" => Some(SysTopicMetadata {
            label: "Stored messages",
            description: "The number of messages currently held in the message store. This includes retained messages and messages queued for durable clients.",
            window: None,
        }),
        "$SYS/broker/store/messages/count" => Some(SysTopicMetadata {
            label: "Stored messages count",
            description: "The number of messages currently held in the message store. This includes retained messages and messages queued for durable clients.",
            window: None,
        }),
        "$SYS/broker/store/messages/bytes" => Some(SysTopicMetadata {
            label: "Stored bytes",
            description: "Bytes currently held in the message store.",
            window: None,
        }),
        _ => aggregate_metadata(topic),
    }
}

fn aggregate_metadata(topic: &str) -> Option<SysTopicMetadata<'_>> {
    let (prefix, window) = topic.rsplit_once('/')?;
    let window = match window {
        "1min" | "5min" | "15min" => window,
        _ => return None,
    };

    let (label, description) = match prefix {
        "$SYS/broker/load/messages/received" => (
            "Aggregated messages received",
            "Broker message receive rate over the reporting window.",
        ),
        "$SYS/broker/load/messages/sent" => (
            "Aggregated messages sent",
            "Broker message send rate over the reporting window.",
        ),
        "$SYS/broker/load/publish/received" => (
            "Aggregated publishes received",
            "Broker publish receive rate over the reporting window.",
        ),
        "$SYS/broker/load/publish/sent" => (
            "Aggregated publishes sent",
            "Broker publish send rate over the reporting window.",
        ),
        "$SYS/broker/load/publish/dropped" => (
            "Aggregated publish dropped",
            "Aggregation: 1min / 5min / 15min",
        ),
        "$SYS/broker/load/bytes/received" => (
            "Aggregated received bytes",
            "Broker byte receive rate over the reporting window.",
        ),
        "$SYS/broker/load/bytes/sent" => (
            "Aggregated sent bytes",
            "Broker byte send rate over the reporting window.",
        ),
        "$SYS/broker/load/sockets" => ("Aggregated sockets", "Aggregation: 1min / 5min / 15min"),
        "$SYS/broker/load/connections" => {
            ("Aggregated connections", "Aggregation: 1min / 5min / 15min")
        }
        _ => return None,
    };

    Some(SysTopicMetadata {
        label,
        description,
        window: Some(window),
    })
}
