package org.correomqtt.gui.helper;

import com.hivemq.client.mqtt.datatypes.MqttTopicFilter;
import javafx.scene.control.ComboBox;
import javafx.scene.control.Tooltip;

public class CheckTopicHelper {

    private static final String TOPIC_IS_EMPTY = "Topic is empty";
    private static final String WILDCARDS_ARE_MISPLACED = "Wildcards are misplaced.";
    private static final String PUBLISH_TOPIC_MUST_NOT_CONTAIN_WILDCARDS = "Publish topic must not contain wildcards";
    private static final String TOPIC_IS_SYS_TOPIC = "Topics starting with $SYS are reserved.";
    public static final String EMPTY_ERROR = "emptyError";
    public static final String EXCLAMATION_CIRCLE_SOLID = "exclamationCircleSolid";

    private CheckTopicHelper(){
        // empty constructor
    }

    public static boolean checkPublishTopic(ComboBox<String> comboBox, boolean save) {
        if (noCheckRequired(comboBox)) {
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

        comboBox.getEditor().getStyleClass().removeAll(EMPTY_ERROR);
        comboBox.getEditor().getStyleClass().removeAll(EXCLAMATION_CIRCLE_SOLID);
        return true;
    }

    public static boolean checkSubscribeTopic(ComboBox<String> comboBox, boolean save, boolean afterSubscribe) {
        if (afterSubscribe) {
            return true;
        }

        if (noCheckRequired(comboBox)) {
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

        comboBox.getEditor().getStyleClass().removeAll(EMPTY_ERROR);
        comboBox.getEditor().getStyleClass().removeAll(EXCLAMATION_CIRCLE_SOLID);
        return true;
    }

    private static void setError(ComboBox<String> comboBox, boolean save, String tooltipText) {
        if (save) {
            comboBox.getEditor().getStyleClass().add(EMPTY_ERROR);
        }

        comboBox.setTooltip(new Tooltip(tooltipText));
        comboBox.getEditor().getStyleClass().add(EXCLAMATION_CIRCLE_SOLID);
    }

    private static boolean noCheckRequired(ComboBox<String> comboBox) {
        return comboBox.getEditor().getText() == null || comboBox.getEditor().getText().isEmpty();
    }
}
