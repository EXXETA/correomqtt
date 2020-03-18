package org.correomqtt.gui.helper;

import com.hivemq.client.mqtt.datatypes.MqttTopicFilter;
import javafx.scene.control.ComboBox;
import javafx.scene.control.Tooltip;

public class CheckTopicHelper {

    private static final String TOPIC_IS_EMPTY = "Topic is empty";
    private static final String WILDCARDS_ARE_MISPLACED = "Wildcards are misplaced.";
    private static final String PUBLISH_TOPIC_MUST_NOT_CONTAIN_WILDCARDS = "Publish topic must not contain wildcards";
    private static final String TOPIC_IS_SYS_TOPIC = "Topics starting with $SYS are reserved.";

    public static boolean checkPublishTopic(ComboBox<String> comboBox, boolean save) {
        if (!checkRequired(comboBox)) {
            setError(comboBox, save, TOPIC_IS_EMPTY);
            return false;
        }

        if (comboBox.getEditor().getText().startsWith("$SYS")) {
            setError(comboBox, save, TOPIC_IS_SYS_TOPIC);
            return false;
        }

        try {
            if (MqttTopicFilter.of(comboBox.getEditor().getText()).containsWildcards()) {
                setError(comboBox, save, PUBLISH_TOPIC_MUST_NOT_CONTAIN_WILDCARDS);
                return false;
            }
        } catch (IllegalArgumentException e) {
            setError(comboBox, save, PUBLISH_TOPIC_MUST_NOT_CONTAIN_WILDCARDS);
            return false;
        }

        comboBox.getEditor().getStyleClass().removeAll("emptyError");
        comboBox.getEditor().getStyleClass().removeAll("exclamationCircleSolid");
        return true;
    }

    public static boolean checkSubscribeTopic(ComboBox<String> comboBox, boolean save, boolean afterSubscribe) {
        if (afterSubscribe) {
            return true;
        }

        if (!checkRequired(comboBox)) {
            setError(comboBox, save, TOPIC_IS_EMPTY);
            return false;
        }

        if (comboBox.getEditor().getText().startsWith("$SYS")) {
            setError(comboBox, save, TOPIC_IS_SYS_TOPIC);
            return false;
        }

        try {
            MqttTopicFilter.of(comboBox.getEditor().getText());
        } catch (IllegalArgumentException e) {
            setError(comboBox, save, WILDCARDS_ARE_MISPLACED);
            return false;
        }

        comboBox.getEditor().getStyleClass().removeAll("emptyError");
        comboBox.getEditor().getStyleClass().removeAll("exclamationCircleSolid");
        return true;
    }

    private static void setError(ComboBox<String> comboBox, boolean save, String tooltipText) {
        if (save) {
            comboBox.getEditor().getStyleClass().add("emptyError");
        }

        comboBox.setTooltip(new Tooltip(tooltipText));
        comboBox.getEditor().getStyleClass().add("exclamationCircleSolid");
    }

    private static boolean checkRequired(ComboBox<String> comboBox) {
        return !(comboBox.getEditor().getText() == null || comboBox.getEditor().getText().isEmpty());
    }
}
