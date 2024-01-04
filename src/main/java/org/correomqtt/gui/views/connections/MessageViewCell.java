package org.correomqtt.gui.views.connections;

import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.control.Tooltip;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Pane;
import org.correomqtt.business.model.LabelType;
import org.correomqtt.business.model.MessageListViewConfig;
import org.correomqtt.business.model.RetainedState;
import org.correomqtt.business.fileprovider.SettingsProvider;
import org.correomqtt.gui.model.MessagePropertiesDTO;
import org.correomqtt.plugin.manager.MessageValidator;
import org.correomqtt.plugin.manager.PluginManager;
import org.correomqtt.plugin.model.MessageExtensionDTO;
import org.correomqtt.plugin.spi.MessageListHook;
import org.correomqtt.plugin.spi.MessageValidatorHook;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.time.format.DateTimeFormatter;
import java.util.ResourceBundle;
import java.util.function.Supplier;

@SuppressWarnings("java:S110")
public class MessageViewCell extends ListCell<MessagePropertiesDTO> {

    public static final DateTimeFormatter FORMATTER = DateTimeFormatter.ofPattern("yyyy-MM-dd'T'HH:mm:ss.SSS'Z'");

    private static final Logger LOGGER = LoggerFactory.getLogger(MessageViewCell.class);
    private static final int MAX_PAYLOAD_LENGTH = 1000;

    private final ListView<MessagePropertiesDTO> listView;
    private final Supplier<MessageListViewConfig> listViewConfigGetter;

    @SuppressWarnings("unused")
    @FXML
    private Pane mainNode;

    @SuppressWarnings("unused")
    @FXML
    private Label topicLabel;

    @SuppressWarnings("unused")
    @FXML
    private HBox labelBox;

    @SuppressWarnings("unused")
    @FXML
    private Label validLabel;

    @SuppressWarnings("unused")
    @FXML
    private Label invalidLabel;

    @SuppressWarnings("unused")
    @FXML
    private Label retainedLabel;

    @SuppressWarnings("unused")
    @FXML
    private Label qosLabel;

    @SuppressWarnings("unused")
    @FXML
    private Label payloadLabel;

    @FXML
    private Label subscriptionLabel;

    @FXML
    private Label timestampLabel;

    private FXMLLoader loader;


    @FXML
    private ResourceBundle resources;

    public MessageViewCell(ListView<MessagePropertiesDTO> listView, Supplier<MessageListViewConfig> listViewConfigGetter) {
        this.listView = listView;
        this.listViewConfigGetter = listViewConfigGetter;
    }

    @Override
    protected void updateItem(MessagePropertiesDTO messageDTO, boolean empty) {
        super.updateItem(messageDTO, empty);
        if (empty || messageDTO == null) {
            setText(null);
            setGraphic(null);
        } else {

            if (loader == null) {
                try {
                    loader = new FXMLLoader(MessageViewCell.class.getResource("messageView.fxml"),
                            ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale()));
                    loader.setController(this);
                    loader.load();

                } catch (Exception e) {
                    LOGGER.error("Exception receiving message:", e);
                    setText(resources.getString("commonRowCreationError"));
                    setGraphic(null);
                    return;
                }

            }
            mainNode.prefWidthProperty().bind(listView.widthProperty().subtract(20));
            setUpMessage(messageDTO);
            setText(null);
            setGraphic(mainNode);
        }
    }

    private void setUpMessage(MessagePropertiesDTO messageDTO) {
        topicLabel.getStyleClass().removeAll("published", "succeeded", "failed");

        if (messageDTO.getPublishStatus() != null) {
            switch (messageDTO.getPublishStatus()) {
                case PUBLISHED:
                    topicLabel.getStyleClass().add("published");
                    break;
                case SUCCEEDED:
                    topicLabel.getStyleClass().add("succeeded");
                    break;
                case FAILED:
                    topicLabel.getStyleClass().add("failed");
                    break;
            }
        }

        executeOnCreateMessageEntryExtensions(messageDTO);

        validateMessage(messageDTO);

        subscriptionLabel.setVisible(false);
        subscriptionLabel.setManaged(false);

        topicLabel.setText(messageDTO.getTopic());

        if (messageDTO.getSubscription() != null) {
            subscriptionLabel.setVisible(true);
            subscriptionLabel.setManaged(true);
            subscriptionLabel.setText(messageDTO.getSubscription().getTopic());
        }

        if(listViewConfigGetter.get().isVisible(LabelType.RETAINED)){
            retainedLabel.setText(messageDTO.isRetained() ? RetainedState.RETAINED.name() : RetainedState.NOT_RETAINED.name());
            retainedLabel.setVisible(true);
            retainedLabel.setManaged(true);
        }else{
            retainedLabel.setVisible(false);
            retainedLabel.setManaged(false);
        }

        if(listViewConfigGetter.get().isVisible(LabelType.QOS)){
            qosLabel.setText(messageDTO.getQos().toString());
            qosLabel.setVisible(true);
            qosLabel.setManaged(true);
        }else{
            qosLabel.setVisible(false);
            qosLabel.setManaged(false);
        }

        if(listViewConfigGetter.get().isVisible(LabelType.TIMESTAMP)){
            timestampLabel.setText(messageDTO.getDateTime().format(FORMATTER));
            timestampLabel.setVisible(true);
            timestampLabel.setManaged(true);
        }else{
            timestampLabel.setVisible(false);
            timestampLabel.setManaged(false);
        }

        String payload = messageDTO.getPayload();
        payloadLabel.setText(payload.substring(0, Math.min(payload.length(), MAX_PAYLOAD_LENGTH))
                .replace("\n", " ")
                .replace("\r", " "
                ).trim());
    }

    private void executeOnCreateMessageEntryExtensions(MessagePropertiesDTO messageDTO) {
        labelBox.getChildren().clear();
        PluginManager.getInstance().getExtensions(MessageListHook.class)
                     .forEach(p -> p.onCreateEntry(new MessageExtensionDTO(messageDTO), labelBox));
    }

    private void validateMessage(MessagePropertiesDTO messageDTO) {
        validLabel.setVisible(false);
        validLabel.setManaged(false);
        invalidLabel.setVisible(false);
        invalidLabel.setManaged(false);

        MessageValidatorHook.Validation validation = MessageValidator.validateMessage(messageDTO.getTopic(), messageDTO.getPayload());
        if (validation != null) {
            updateValidatorLabel(validLabel, validation.isValid(), validation.getTooltip());
            updateValidatorLabel(invalidLabel, !validation.isValid(), validation.getTooltip());
        }
    }

    private void updateValidatorLabel(Label label, boolean isVisible, String tooltip) {
        label.setVisible(isVisible);
        label.setManaged(isVisible);
        label.setTooltip(new Tooltip(tooltip));
    }
}
