package org.correomqtt.gui.controller;

import org.correomqtt.business.exception.CorreoMqttException;
import org.correomqtt.business.model.*;
import org.correomqtt.business.provider.PersistPublishHistoryProvider;
import org.correomqtt.business.provider.PersistPublishMessageHistoryProvider;
import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.gui.business.TaskFactory;
import org.correomqtt.gui.cell.QosCell;
import org.correomqtt.gui.cell.TopicCell;
import org.correomqtt.gui.helper.AlertHelper;
import org.correomqtt.gui.helper.CheckTopicHelper;
import org.correomqtt.gui.model.MessagePropertiesDTO;
import org.correomqtt.gui.transformer.MessageTransformer;
import org.correomqtt.plugin.manager.PluginManager;
import org.correomqtt.plugin.model.MessageExtensionDTO;
import org.correomqtt.plugin.spi.MessageContextMenuHook;
import org.correomqtt.plugin.spi.PublishMenuHook;
import javafx.application.Platform;
import javafx.collections.FXCollections;
import javafx.fxml.FXML;
import javafx.geometry.Pos;
import javafx.scene.control.Button;
import javafx.scene.control.CheckBox;
import javafx.scene.control.ComboBox;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Pane;
import javafx.stage.FileChooser;
import javafx.stage.Stage;
import org.correomqtt.business.dispatcher.*;
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

public class PublishViewController extends BaseMessageBasedViewController implements ConnectionLifecycleObserver,
        PublishObserver,
        ConfigObserver,
        ShortcutObserver,
        ImportMessageObserver,
        PersistPublishHistoryObserver {

    private static final Logger LOGGER = LoggerFactory.getLogger(PublishViewController.class);
    private static ResourceBundle resources;
    private final PublishViewDelegate delegate;
    private final PluginManager pluginSystem = PluginManager.getInstance();

    @FXML
    public AnchorPane publishViewAnchor;

    @FXML
    public ComboBox<Qos> qosComboBox;

    @FXML
    public ComboBox<String> topicComboBox;

    @FXML
    public HBox pluginControlBox;

    @FXML
    public CheckBox retainedCheckBox;

    @FXML
    public Button publishButton;

    @FXML
    private CodeArea payloadCodeArea;

    @FXML
    private Pane codeAreaScrollPane;

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

        pluginSystem.getExtensions(PublishMenuHook.class).forEach(p -> {
            HBox pluginBox = new HBox();
            pluginBox.setAlignment(Pos.CENTER_RIGHT);
            pluginControlBox.getChildren().add(pluginBox);
            p.onInstantiatePublishMenu(getConnectionId(), pluginBox);
        });

        topicComboBox.getEditor().lengthProperty().addListener(((observable, oldValue, newValue) -> CheckTopicHelper.checkPublishTopic(topicComboBox, false)));

        codeAreaScrollPane.getChildren().add(new VirtualizedScrollPane<>(payloadCodeArea));
        payloadCodeArea.prefWidthProperty().bind(codeAreaScrollPane.widthProperty());
        payloadCodeArea.prefHeightProperty().bind(codeAreaScrollPane.heightProperty());

        SettingsProvider.getInstance().getConnectionConfigs().stream()
                .filter(c -> c.getId().equals(getConnectionId()))
                .findFirst()
                .ifPresent(c -> {
                    splitPane.getDividers().get(0).setPosition(c.getConnectionUISettings().getPublishDividerPosition());
                    super.messageListViewController.showDetailViewButton.setSelected(c.getConnectionUISettings().isPublishDetailActive());
                    if (c.getConnectionUISettings().isPublishDetailActive()) {
                        super.messageListViewController.showDetailView();
                        super.messageListViewController.splitPane.getDividers().get(0).setPosition(c.getConnectionUISettings().getPublishDetailDividerPosition());
                    }
                });

        initTopicComboBox();
    }

    private void initTopicComboBox() {
        List<String> topics = PersistPublishHistoryProvider.getInstance(getConnectionId()).getTopics(getConnectionId());
        topicComboBox.setItems(FXCollections.observableArrayList(topics));
        topicComboBox.setCellFactory(TopicCell::new);
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
        retainedCheckBox.setSelected(false);

        // reverse order, because first message in history must be last one to add
        new LinkedList<>(PersistPublishMessageHistoryProvider.getInstance(getConnectionId())
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
        executeOnCopyMessageToFormExtensions(messageDTO);

        // Retained-Abfrage
        if (messageDTO.isRetained()) {
            retainedCheckBox.setSelected(true);
        } else {
            retainedCheckBox.setSelected(false);
        }
        payloadCodeArea.replaceText(messageDTO.getPayload());
        topicComboBox.setValue(messageDTO.getTopic());

        qosComboBox.setValue(messageDTO.getQos());

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Message copied to form: {}", getConnectionId());
        }
    }

    private void executeOnCopyMessageToFormExtensions(MessagePropertiesDTO messageDTO) {
        pluginSystem.getExtensions(MessageContextMenuHook.class)
                .forEach(p -> p.onCopyMessageToPublishForm(getConnectionId(), new MessageExtensionDTO(messageDTO)));
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
    public void onSettingsUpdated(boolean showRestartRequiredDialog) {

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
