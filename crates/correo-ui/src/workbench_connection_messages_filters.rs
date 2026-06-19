use correo_core::{AppSnapshot, MessageRow};

pub(crate) fn row_matches(topic: &str, payload_preview: &str, filter: &str) -> bool {
    filter.is_empty()
        || topic.to_ascii_lowercase().contains(filter)
        || payload_preview.to_ascii_lowercase().contains(filter)
}

pub(crate) fn message_visible_for_subscriptions(
    message: &MessageRow,
    snapshot: &AppSnapshot,
) -> bool {
    let subscriptions = &snapshot.workbench.subscribe.subscriptions;
    subscriptions.is_empty()
        || subscriptions.iter().any(|subscription| {
            subscription.messages_visible
                && topic_matches_filter(&message.topic, &subscription.topic_filter)
        })
}

pub(crate) fn topic_matches_filter(topic: &str, filter: &str) -> bool {
    let mut topic_segments = topic.split('/').peekable();
    let mut filter_segments = filter.split('/').peekable();
    while let Some(filter_segment) = filter_segments.next() {
        match filter_segment {
            "#" => return filter_segments.peek().is_none(),
            "+" if topic_segments.next().is_none() => return false,
            "+" => {}
            segment if topic_segments.next() != Some(segment) => return false,
            _ => {}
        }
    }
    topic_segments.next().is_none()
}
