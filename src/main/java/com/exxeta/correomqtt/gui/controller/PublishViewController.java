package com.exxeta.correomqtt.gui.controller;

import com.exxeta.correomqtt.business.dispatcher.ConfigDispatcher;
import com.exxeta.correomqtt.business.dispatcher.ConfigObserver;
import com.exxeta.correomqtt.business.dispatcher.ConnectionLifecycleDispatcher;
import com.exxeta.correomqtt.business.dispatcher.ConnectionLifecycleObserver;
import com.exxeta.correomqtt.business.dispatcher.ImportMessageDispatcher;
import com.exxeta.correomqtt.business.dispatcher.ImportMessageObserver;
import com.exxeta.correomqtt.business.dispatcher.PersistPublishHistoryDispatcher;
import com.exxeta.correomqtt.business.dispatcher.PersistPublishHistoryObserver;
import com.exxeta.correomqtt.business.dispatcher.PublishDispatcher;
import com.exxeta.correomqtt.business.dispatcher.PublishGlobalDispatcher;
import com.exxeta.correomqtt.business.dispatcher.PublishObserver;
import com.exxeta.correomqtt.business.dispatcher.ShortcutDispatcher;
import com.exxeta.correomqtt.business.dispatcher.ShortcutObserver;
import com.exxeta.correomqtt.business.exception.CorreoMqttException;
import com.exxeta.correomqtt.business.model.MessageDTO;
import com.exxeta.correomqtt.business.model.MessageType;
import com.exxeta.correomqtt.business.model.PublishStatus;
import com.exxeta.correomqtt.business.model.Qos;
import com.exxeta.correomqtt.business.services.ConfigService;
import com.exxeta.correomqtt.business.services.PersistPublishHistoryService;
import com.exxeta.correomqtt.business.services.PersistPublishMessageHistoryService;
import com.exxeta.correomqtt.gui.business.TaskFactory;
import com.exxeta.correomqtt.gui.cell.QosCell;
import com.exxeta.correomqtt.gui.cell.TopicCell;
import com.exxeta.correomqtt.gui.helper.AlertHelper;
import com.exxeta.correomqtt.gui.helper.CheckTopicHelper;
import com.exxeta.correomqtt.gui.model.MessagePropertiesDTO;
import com.exxeta.correomqtt.gui.transformer.MessageTransformer;
import javafx.application.Platform;
import javafx.collections.FXCollections;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.CheckBox;
import javafx.scene.control.ComboBox;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import javafx.stage.FileChooser;
import javafx.stage.Stage;
import org.fxmisc.flowless.VirtualizedScrollPane;
import org.fxmisc.richtext.CodeArea;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.time.LocalDateTime;
import java.util.LinkedList;
import java.util.List;
import java.util.ResourceBundle;
import java.util.UUID;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

public class PublishViewController extends BaseMessageBasedViewController implements ConnectionLifecycleObserver,
        PublishObserver,
        ConfigObserver,
        ShortcutObserver,
        ImportMessageObserver,
        PersistPublishHistoryObserver {

    private static final Logger LOGGER = LoggerFactory.getLogger(PublishViewController.class);
    private static ResourceBundle resources;
    private final PublishViewDelegate delegate;

    @FXML
    public AnchorPane publishViewAnchor;

    @FXML
    public ComboBox<Qos> qosComboBox;

    @FXML
    public ComboBox<String> topicComboBox;

    @FXML
    public CheckBox messageIdCheckBox;

    @FXML
    public CheckBox answerExpectedCheckBox;

    @FXML
    public CheckBox retainedCheckBox;

    @FXML
    public Button publishButton;

    @FXML
    private CodeArea payloadCodeArea;

    @FXML
    private Pane codeAreaScrollPane;


    //Pattern
    private Pattern pattern = Pattern.compile("^([0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[0-9a-f]{4}-[0-9a-f]{12})([0-1]{1})", Pattern.CASE_INSENSITIVE);

    private LoadingViewController loadingViewController;

    public PublishViewController(String connectionId, PublishViewDelegate delegate) {
        super(connectionId);
        this.delegate = delegate;
        ConnectionLifecycleDispatcher.getInstance().addObserver(this);
        PublishDispatcher.getInstance().addObserver(this);
        ConfigDispatcher.getInstance().addObserver(this);
        ShortcutDispatcher.getInstance().addObserver(this);
        ImportMessageDispatcher.getInstance().addObserver(this);
        PersistPublishHistoryDispatcher.getInstance().addObserver(this);
    }

    static LoaderResult<PublishViewController> load(String connectionId, PublishViewDelegate delegate) {
        LoaderResult<PublishViewController> result = load(PublishViewController.class, "publishView.fxml",
                () -> new PublishViewController(connectionId, delegate));
        resources = result.getResourceBundle();
        return result;
    }

    @FXML
    public void initialize() {
        initMessageListView();

        qosComboBox.setItems(FXCollections.observableArrayList(Qos.values()));
        qosComboBox.getSelectionModel().selectFirst();
        qosComboBox.setCellFactory(QosCell::new);

        if (!ConfigService.getInstance().getSettings().isExtraFeatures()) {
            messageIdCheckBox.setVisible(false);
            answerExpectedCheckBox.setVisible(false);
        }

        topicComboBox.getEditor().lengthProperty().addListener(((observable, oldValue, newValue) -> CheckTopicHelper.checkPublishTopic(topicComboBox, false)));

        codeAreaScrollPane.getChildren().add(new VirtualizedScrollPane<>(payloadCodeArea));
        payloadCodeArea.prefWidthProperty().bind(codeAreaScrollPane.widthProperty());
        payloadCodeArea.prefHeightProperty().bind(codeAreaScrollPane.heightProperty());

        initTopicComboBox();
    }

    private void initTopicComboBox() {
        List<String> topics = PersistPublishHistoryService.getInstance(getConnectionId()).getTopics(getConnectionId());
        topicComboBox.setItems(FXCollections.observableArrayList(topics));
        topicComboBox.setCellFactory(TopicCell::new);
    }

    public void setCheckBoxes() {
        if (messageIdCheckBox.isSelected()) {
            answerExpectedCheckBox.setDisable(false);
        } else {
            answerExpectedCheckBox.setDisable(true);
            answerExpectedCheckBox.setSelected(false);
        }
    }

    @FXML
    public void onClickPublishKey(KeyEvent actionEvent) {
        if (actionEvent.getCode() == KeyCode.ENTER) {
            topicComboBox.setValue(topicComboBox.getEditor().getText());
            if (topicComboBox.getValue() == null) {
                return;
            }
            publish();

            if (LOGGER.isDebugEnabled()) {
                LOGGER.debug("Publish button clicked.");
            }
        }
    }

    @FXML
    private void publish() {
        if (!CheckTopicHelper.checkPublishTopic(topicComboBox, true)) {
            return;
        }

        MessagePropertiesDTO messagePropertiesDTO = MessagePropertiesDTO.builder()
                                                                        .topic(topicComboBox.getValue())
                                                                        .qos(qosComboBox.getSelectionModel().getSelectedItem())
                                                                        .isRetained(retainedCheckBox.isSelected())
                                                                        .payload(payloadCodeArea.getText())
                                                                        .messageId(UUID.randomUUID().toString())
                                                                        .messageType(MessageType.OUTGOING)
                                                                        .dateTime(LocalDateTime.now())
                                                                        .build();

        if (ConfigService.getInstance().getSettings().isExtraFeatures() && messageIdCheckBox.isSelected()) {
            if (answerExpectedCheckBox.isSelected()) {
                messagePropertiesDTO.setPayload("1" + messagePropertiesDTO.getPayload());
            } else {
                messagePropertiesDTO.setPayload("0" + messagePropertiesDTO.getPayload());
            }

            String uuid = UUID.randomUUID().toString();
            messagePropertiesDTO.setPayload(uuid + messagePropertiesDTO.getPayload());
            messagePropertiesDTO.getSpecialMessageIdProperty().set(uuid);
            messagePropertiesDTO.getAnswerExpectedProperty().set(answerExpectedCheckBox.isSelected());
        }


        TaskFactory.publish(getConnectionId(), messagePropertiesDTO);

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Publishing to topic: {}: {}", messagePropertiesDTO.getTopic(), getConnectionId());
        }
    }

    @FXML
    public void onClickScan() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Open file button clicked: {}", getConnectionId());
        }

        Stage stage = new Stage();
        stage.setAlwaysOnTop(true);
        final FileChooser fileChooser = new FileChooser();
        fileChooser.setTitle(resources.getString("publishViewControllerOpenFileTitle"));
        File file = fileChooser.showOpenDialog(stage);

        if (file != null) {
            TaskFactory.importMessage(getConnectionId(), file);
        }
    }

    @Override
    public void onDisconnectFromConnectionDeleted(String connectionId) {
        // do nothing
    }

    @Override
    public void onConnect() {
        topicComboBox.valueProperty().set(null);
        payloadCodeArea.replaceText("");
        messageIdCheckBox.setSelected(false);
        answerExpectedCheckBox.setSelected(false);
        retainedCheckBox.setSelected(false);

        // reverse order, because first message in history must be last one to add
        new LinkedList<>(PersistPublishMessageHistoryService.getInstance(getConnectionId())
                                                            .getMessages(getConnectionId()))
                .descendingIterator()
                .forEachRemaining(messageDTO -> messageListViewController.onNewMessage(MessageTransformer.dtoToProps(messageDTO)));
    }

    @Override
    public void onConnectRunning() {
        // nothing to do
    }

    @Override
    public void onConnectionFailed(Throwable message) {
        // nothing to do
    }

    @Override
    public void onConnectionCanceled() {
        // nothing to do
    }

    @Override
    public void onConnectionLost() {
        // nothing to do
    }

    @Override
    public void onDisconnect() {
        // nothing to do
    }

    @Override
    public void onDisconnectCanceled() {
        // nothing to do
    }

    @Override
    public void onDisconnectFailed(Throwable exception) {
        // nothing to do
    }

    @Override
    public void onDisconnectRunning() {
        // nothing to do
    }

    @Override
    public void onConnectionReconnected() {
        // nothing to do
    }

    @Override
    public void onReconnectFailed(AtomicInteger triedReconnects, int maxReconnects) {
        // nothing to do
    }

    @Override
    public void setUpToForm(MessagePropertiesDTO messageDTO) {
        String finalMessage = messageDTO.getPayload();

        Matcher m = pattern.matcher(finalMessage);
        if (m.find()) {
            boolean answerExpected = "1".equals(m.group(2));
            messageIdCheckBox.setSelected(true);
            answerExpectedCheckBox.setDisable(false);
            if (answerExpected) {
                answerExpectedCheckBox.setSelected(true);
            } else {
                answerExpectedCheckBox.setSelected(false);
            }
            finalMessage = finalMessage.substring(37);
        } else {
            messageIdCheckBox.setSelected(false);
            answerExpectedCheckBox.setDisable(true);
            answerExpectedCheckBox.setSelected(false);
        }

        // Retained-Abfrage
        if (messageDTO.isRetained()) {
            retainedCheckBox.setSelected(true);
        } else {
            retainedCheckBox.setSelected(false);
        }
        payloadCodeArea.replaceText(finalMessage);
        topicComboBox.setValue(messageDTO.getTopic());

        qosComboBox.setValue(messageDTO.getQos());

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Message copied to form: {}", getConnectionId());
        }
    }

    @Override
    public void onConfigDirectoryEmpty() {
        // nothing to do
    }

    @Override
    public void onConfigDirectoryNotAccessible() {
        // nothing to do
    }

    @Override
    public void onAppDataNull() {
        // nothing to do
    }

    @Override
    public void onUserHomeNull() {
        // nothing to do
    }

    @Override
    public void onFileAlreadyExists() {
        // nothing to do
    }

    @Override
    public void onInvalidPath() {
        // nothing to do
    }

    @Override
    public void onInvalidJsonFormat() {
        // nothing to do
    }

    @Override
    public void onSavingFailed() {
        // nothing to do
    }

    @Override
    public void onSettingsUpdated() {
        if (!ConfigService.getInstance().getSettings().isExtraFeatures()) {
            messageIdCheckBox.setVisible(false);
            messageIdCheckBox.setSelected(false);
            answerExpectedCheckBox.setVisible(false);
            answerExpectedCheckBox.setSelected(false);
        } else {
            messageIdCheckBox.setVisible(true);
            answerExpectedCheckBox.setVisible(true);
        }

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Updated settings in publish view controller: {}", getConnectionId());
        }
    }

    @Override
    public void onConnectionsUpdated() {
        // nothing to do
    }

    @Override
    public void onConfigPrepareFailed() {
        // nothing to do
    }

    @Override
    public void onPublishShortcutPressed() {
        // nothing to do
    }

    @Override
    public void onClearOutgoingShortcutPressed() {
        // nothing to do
    }

    @Override
    public void onPublishSucceeded(MessageDTO messageDTO) {
        messageDTO.setPublishStatus(PublishStatus.SUCCEEDED);
        messageListViewController.onNewMessage(MessageTransformer.dtoToProps(messageDTO));
    }

    @Override
    public void onPublishCancelled(MessageDTO messageDTO) {
        messageDTO.setPublishStatus(PublishStatus.FAILED);
        messageListViewController.onNewMessage(MessageTransformer.dtoToProps(messageDTO));
    }

    @Override
    public void onPublishFailed(MessageDTO messageDTO, Throwable exception) {
        messageDTO.setPublishStatus(PublishStatus.FAILED);
        messageListViewController.onNewMessage(MessageTransformer.dtoToProps(messageDTO));

        String msg;
        if (exception instanceof CorreoMqttException) {
            msg = ((CorreoMqttException) exception).getInfo();
        } else {
            msg = "Exception in business layer: " + exception.getMessage();
        }
        AlertHelper.warn(resources.getString("publishViewControllerPublishFailedTitle"),
                resources.getString("publishViewControllerPublishFailedContent") + ": " + messageDTO.getTopic() + ": " + msg);
    }

    @Override
    public void onPublishRunning(MessageDTO messageDTO) {
        // nothing to do
    }

    @Override
    public void onPublishScheduled(MessageDTO messageDTO) {
        messageDTO.setPublishStatus(PublishStatus.PUBLISEHD);
        messageListViewController.onNewMessage(MessageTransformer.dtoToProps(messageDTO));
    }

    @Override
    public void onImportStarted(File file) {
        Platform.runLater(() -> {
            loadingViewController = LoadingViewController.showAsDialog(getConnectionId(),
                    resources.getString("publishViewControllerOpenFileTitle") + ": " + file.getAbsolutePath());
            loadingViewController.setProgress(1);
        });
    }

    @Override
    public void onImportSucceeded(MessageDTO messageDTO) {
        Platform.runLater(() -> {
            topicComboBox.setValue(messageDTO.getTopic());
            retainedCheckBox.setSelected(messageDTO.isRetained());
            if (ConfigService.getInstance().getSettings().isExtraFeatures()) {
                // TODO no extra features in MessageDTO yet
            }
            qosComboBox.setValue(messageDTO.getQos());
            payloadCodeArea.replaceText(messageDTO.getPayload());
            if (loadingViewController != null) {
                loadingViewController.close();
                loadingViewController = null;
            }
        });
    }

    @Override
    public void onImportCancelled(File file) {
        Platform.runLater(() -> {
            if (loadingViewController != null) {
                loadingViewController.close();
                loadingViewController = null;
            }
            AlertHelper.warn(resources.getString("publishViewControllerImportCancelledTitle"),
                             resources.getString("publishViewControllerImportFileCancelledContent"));
        });

    }

    @Override
    public void onImportFailed(File file, Throwable exception) {
        Platform.runLater(() -> {
            if (loadingViewController != null) {
                loadingViewController.close();
                loadingViewController = null;
            }
            AlertHelper.warn(resources.getString("publishViewControllerImportFileFailedTitle"),
                              resources.getString("publishViewControllerImportFileFailedContent"));
        });
    }

    @Override
    public void removeMessage(MessageDTO messageDTO) {
        PublishGlobalDispatcher.getInstance().onPublishRemoved(getConnectionId(), messageDTO);
    }

    @Override
    public void clearMessages() {
        PublishGlobalDispatcher.getInstance().onPublishesCleared(getConnectionId());
    }

    @Override
    public void setTabDirty() {
        delegate.setTabDirty();
    }

    @Override
    public void errorReadingPublishHistory(Throwable exception) {

    }

    @Override
    public void errorWritingPublishHistory(Throwable exception) {

    }

    @Override
    public void updatedPublishes(String connectionId) {
        initTopicComboBox();
    }
}
