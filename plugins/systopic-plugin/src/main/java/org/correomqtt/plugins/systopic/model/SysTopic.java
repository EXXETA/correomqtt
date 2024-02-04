package org.correomqtt.plugins.systopic.model;

import java.util.Arrays;
import java.util.HashMap;
import java.util.Map;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

public enum SysTopic {
    VERSION("$SYS/broker/version", "sysTopicEnumVersionTranslation", "sysTopicEnumVersionDescription"),
    UPTIME("$SYS/broker/uptime", "sysTopicEnumUptimeTranslation", "sysTopicEnumUptimeDescription"),
    TIME("$SYS/broker/time", "sysTopicEnumTimeTranslation", "sysTopicEnumTimeDescription"),
    CHANGESET("$SYS/broker/changeset", "sysTopicEnumChangesetTranslation", "sysTopicEnumChangesetDescription"),

    COUNT_RETAINED("$SYS/broker/messages/retained/count", "sysTopicEnumCountRetainedTranslation", "sysTopicEnumCountRetainedDescription"),
    RETAINED_SPECIAL("$SYS/broker/retained messages/count", "sysTopicEnumRetainedSpecialTranslation", "sysTopicEnumRetainedSpecialDescription"),
    COUNT_SUBSCRIPTIONS("$SYS/broker/subscriptions/count", "sysTopicEnumCountSubscriptionsTranslation", "sysTopicEnumCountSubscriptionsDescription"),
    CONNECTED("$SYS/broker/clients/connected", "sysTopicEnumConnectedTranslation", "sysTopicEnumConnectedDescription"),
    CONNECTED_ACTIVE("$SYS/broker/clients/active", "sysTopicEnumConnectedActiveTranslation", "sysTopicEnumConnectedActiveDescription"),
    DISCONNECTED("$SYS/broker/clients/disconnected", "sysTopicEnumDisconnectedTranslation", "sysTopicEnumDisconnectedDescription"),
    MAXIMUM("$SYS/broker/clients/maximum", "sysTopicEnumMaximumTranslation", "sysTopicEnumMaximumDescription"),
    TOTAL("$SYS/broker/clients/total", "sysTopicEnumTotalTranslation", "sysTopicEnumTotalDescription"),
    RECEIVED_MESSAGES("$SYS/broker/messages/received", "sysTopicEnumReceivedMessagesTranslation", "sysTopicEnumReceivedMessagesDescription"),
    SENT_MESSAGES("$SYS/broker/messages/sent", "sysTopicEnumSentMessagesTranslation", "sysTopicEnumSentMessagesDescription"),
    DROPPED("$SYS/broker/messages/publish/dropped", "sysTopicEnumDroppedTranslation", "sysTopicEnumDroppedDescription"),
    RECEIVED_PUBLISH("$SYS/broker/messages/publish/received", "sysTopicEnumReceivedPublishTranslation", "sysTopicEnumReceivedPublishDescription"),
    SENT_PUBLISH("$SYS/broker/messages/publish/sent", "sysTopicEnumSentPublishTranslation", "sysTopicEnumSentPublishDescription"),
    RECEIVED_LOAD_BYTES("$SYS/broker/load/bytes/received", "sysTopicEnumReceivedLoadBytesTranslation", "sysTopicEnumReceivedLoadBytesDescription"),
    SENT_LOAD_BYTES("$SYS/broker/load/bytes/sent", "sysTopicEnumSentLoadBytesTranslation", "sysTopicEnumSentLoadBytesDescription"),
    RECEIVED_BYTES("$SYS/broker/bytes/received", "sysTopicEnumReceivedLoadBytesTranslation", "sysTopicEnumReceivedLoadBytesDescription"),
    SENT_BYTES("$SYS/broker/bytes/sent", "sysTopicEnumSentLoadBytesTranslation", "sysTopicEnumSentLoadBytesDescription"),
    SENT_PUBLISH_BYTES("$SYS/broker/publish/bytes/sent", "sysTopicEnumSentPublishBytesTranslation", "sysTopicEnumSentPublishBytesDescription"),
    INFLIGHT("$SYS/broker/messages/inflight", "sysTopicEnumInflightTranslation", "sysTopicEnumInflightDescription"),
    STORED("$SYS/broker/messages/stored", "sysTopicEnumStoredTranslation", "sysTopicEnumStoredDescription"),
    STORED_COUNT("$SYS/broker/store/messages/count", "sysTopicEnumStoredCountTranslation", "sysTopicEnumStoredCountDescription"),
    STORED_BYTES("$SYS/broker/store/messages/bytes", "sysTopicEnumStoredBytesTranslation", "sysTopicEnumStoredBytesDescription"),
    BUILD_TIME("$SYS/broker/timestamp", "sysTopicBuildTimeTranslation", "sysTopicEnumBuildTimeDescription"),

    RECEIVED_MESSAGES_AGGREGATED("^\\$SYS\\/broker\\/load\\/messages\\/received\\/(1|5|15)min$", "sysTopicEnumReceivedMessagesAggregatedTranslation", "sysTopicEnumAggregationDescription", true),
    SENT_MESSAGES_AGGREGATED("^\\$SYS\\/broker\\/load\\/messages\\/sent\\/(1|5|15)min$", "sysTopicEnumSentMessagesAggregatedTranslation", "sysTopicEnumAggregationDescription", true),
    SOCKETS_AGGREGATED("^\\$SYS\\/broker\\/load\\/sockets\\/(1|5|15)min$", "sysTopicEnumSocketsAggregatedTranslation", "sysTopicEnumAggregationDescription", true),
    CONNECTIONS_AGGREGATED("^\\$SYS\\/broker\\/load\\/connections\\/(1|5|15)min$", "sysTopicEnumConnectionsAggregatedTranslation", "sysTopicEnumAggregationDescription", true),
    PUBLISH_RECEIVED_AGGREGATED("^\\$SYS\\/broker\\/load\\/publish\\/received\\/(1|5|15)min$", "sysTopicEnumPublishReceivedAggregatedTranslation", "sysTopicEnumAggregationDescription", true),
    PUBLISH_SENT_AGGREGATED("^\\$SYS\\/broker\\/load\\/publish\\/sent\\/(1|5|15)min$", "sysTopicEnumPublishSentAggregatedTranslation", "sysTopicEnumAggregationDescription", true),
    PUBLISH_DROPPED_AGGREGATED("^\\$SYS\\/broker\\/load\\/publish\\/dropped\\/(1|5|15)min$", "sysTopicEnumPublishDroppedAggregatedTranslation", "sysTopicEnumAggregationDescription", true),
    BYTES_RECEIVED_AGGREGATED("^\\$SYS\\/broker\\/load\\/bytes\\/received\\/(1|5|15)min$", "sysTopicsEnumBytesReceivedAggregatedTranslation", "sysTopicEnumAggregationDescription", true),
    BYTES_SENT_AGGREGATED("^\\$SYS\\/broker\\/load\\/bytes\\/sent\\/(1|5|15)min$", "sysTopicEnumBytesSentAggregatedTranslation", "sysTopicEnumAggregationDescription", true);

    private static Map<String, Pattern> patternMap = new HashMap<>();

    private static Map<String, SysTopic> lookupMap = new HashMap<>();

    static {
        for ( SysTopic value : values() ) {
            Pattern pattern;
            if (value.aggregated) {
                pattern = Pattern.compile(value.getTopic());
            } else {
                pattern = Pattern.compile("^" + Pattern.quote(value.getTopic()) + "$");
            }
            patternMap.put(value.getTopic(), pattern);
            lookupMap.put(value.getTopic(), value);
        }
    }

    private final String topic;
    private final String translation;
    private final String description;
    private final boolean aggregated;

    SysTopic(String topic, String translation, String description) {
        this.topic = topic;
        this.translation = translation;
        this.description = description;
        this.aggregated = false;
    }

    SysTopic(String topic, String translation, String description, boolean aggregated) {
        this.topic = topic;
        this.translation = translation;
        this.description = description;
        this.aggregated = aggregated;
    }

    public static SysTopic getSysTopicByTopic(String topic) {

        return Arrays.stream(values())
                     .filter(st -> patternMap.get(st.getTopic()).matcher(topic).find())
                     .findFirst()
                     .orElse(null);
    }

    public static int getSortIndex(String topic) {
        int index = 0;
        for ( SysTopic value : SysTopic.values() ) {
            if (value == SysTopic.getSysTopicByTopic(topic)) {
                return index;
            }
            index++;
        }
        return Integer.MAX_VALUE / 2;
    }

    public boolean isAggregated() {
        return aggregated;
    }

    public String getTopic() {
        return topic;
    }

    public String getTranslation() {
        return translation;
    }

    public String getDescription() {
        return description;
    }

    public String parseComponent(String topic) {
        Matcher matcher = patternMap.get(getTopic()).matcher(topic);
        if (matcher.matches()) {
            return matcher.group(1);
        }
        return null;
    }
}
