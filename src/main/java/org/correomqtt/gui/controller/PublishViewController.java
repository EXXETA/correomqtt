package org.correomqtt.gui.controller;

import javafx.application.Platform;
import javafx.beans.value.ChangeListener;
import javafx.collections.FXCollections;
import javafx.fxml.FXML;
import javafx.geometry.Pos;
import javafx.scene.control.Button;
import javafx.scene.control.CheckBox;
import javafx.scene.control.ComboBox;
import javafx.scene.control.ToggleButton;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Pane;
import javafx.stage.FileChooser;
import javafx.stage.Stage;
import org.correomqtt.business.concurrent.ExceptionListener;
import org.correomqtt.business.connection.ConnectEvent;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.eventbus.Subscribe;
import org.correomqtt.business.exception.CorreoMqttException;
import org.correomqtt.business.fileprovider.PersistPublishHistoryProvider;
import org.correomqtt.business.fileprovider.PersistPublishMessageHistoryProvider;
import org.correomqtt.business.fileprovider.SettingsProvider;
import org.correomqtt.business.importexport.messages.ImportMessageFailedEvent;
import org.correomqtt.business.importexport.messages.ImportMessageStartedEvent;
import org.correomqtt.business.importexport.messages.ImportMessageSuccessEvent;
import org.correomqtt.business.importexport.messages.ImportMessageTask;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.ControllerType;
import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.model.MessageListViewConfig;
import org.correomqtt.business.model.MessageType;
import org.correomqtt.business.model.PublishStatus;
import org.correomqtt.business.model.Qos;
import org.correomqtt.business.pubsub.PublishEvent;
import org.correomqtt.business.pubsub.PublishListRemovedEvent;
import org.correomqtt.business.pubsub.PublishTask;
import org.correomqtt.business.pubsub.PublishListClearEvent;
import org.correomqtt.business.utils.AutoFormatPayload;
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
import java.util.function.Supplier;

public class PublishViewController extends BaseMessageBasedViewController {

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

    @FXML
    private ToggleButton publishViewFormatToggleButton;

    private LoadingViewController loadingViewController;
    private ChangeListener<String> payloadCodeAreaChangeListener;

    public PublishViewController(String connectionId, PublishViewDelegate delegate) {
        super(connectionId);
        this.delegate = delegate;
        EventBus.register(this);
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

        payloadCodeAreaChangeListener = (observableValue, s, t1) -> checkFormat();

        publishViewFormatToggleButton.setSelected(true);
        publishViewFormatToggleButton.setOnMouseClicked(mouseEvent -> AutoFormatPayload.autoFormatPayload(
                payloadCodeArea.getText(),
                publishViewFormatToggleButton.isSelected(),
                getConnectionId(),
                payloadCodeArea,
                payloadCodeAreaChangeListener));

        payloadCodeArea.textProperty().addListener(payloadCodeAreaChangeListener);

        SettingsProvider.getInstance().getConnectionConfigs().stream()
                .filter(c -> c.getId().equals(getConnectionId()))
                .findFirst()
                .ifPresent(c -> {
                    if (!splitPane.getDividers().isEmpty()) {
                        splitPane.getDividers().get(0).setPosition(c.getConnectionUISettings().getPublishDividerPosition());
                    }
                    super.messageListViewController.showDetailViewButton.setSelected(c.getConnectionUISettings().isPublishDetailActive());
                    super.messageListViewController.controllerType = ControllerType.PUBLISH;
                    if (c.getConnectionUISettings().isPublishDetailActive()) {
                        super.messageListViewController.showDetailView();
                        if (!super.messageListViewController.splitPane.getDividers().isEmpty()) {
                            super.messageListViewController.splitPane.getDividers().get(0).setPosition(c.getConnectionUISettings().getPublishDetailDividerPosition());
                        }
                    }
                });

        initTopicComboBox();
    }

    private void checkFormat() {
        AutoFormatPayload.autoFormatPayload(payloadCodeArea.getText(), publishViewFormatToggleButton.isSelected(), getConnectionId(), payloadCodeArea, payloadCodeAreaChangeListener);
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

        MessageDTO messageDTO = MessageDTO.builder()
                .topic(topicComboBox.getValue())
                .qos(qosComboBox.getSelectionModel().getSelectedItem())
                .isRetained(retainedCheckBox.isSelected())
                .payload(payloadCodeArea.getText())
                .messageId(UUID.randomUUID().toString())
                .messageType(MessageType.OUTGOING)
                .dateTime(LocalDateTime.now())
                .build();

        new PublishTask(getConnectionId(), messageDTO)
                .onError((ExceptionListener) ex -> this.onPublishFailed(messageDTO, ex))
                .run();

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Publishing to topic: {}: {}", messageDTO.getTopic(), getConnectionId());
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
            new ImportMessageTask(getConnectionId(),file)
                    .run();
            //TODO Direct handler
        }
    }

    @Subscribe(ConnectEvent.class)
    public void onConnect() {
        // reverse order, because first message in history must be last one to add
        new LinkedList<>(PersistPublishMessageHistoryProvider.getInstance(getConnectionId())
                .getMessages(getConnectionId()))
                .descendingIterator()
                .forEachRemaining(messageDTO -> messageListViewController.onNewMessage(MessageTransformer.dtoToProps(messageDTO)));
    }

    @Override
    public void setUpToForm(MessagePropertiesDTO messageDTO) {
        executeOnCopyMessageToFormExtensions(messageDTO);

        // Retained-Abfrage
        retainedCheckBox.setSelected(messageDTO.isRetained());

        payloadCodeArea.replaceText(messageDTO.getPayload());

        checkFormat();

        topicComboBox.setValue(messageDTO.getTopic());

        qosComboBox.setValue(messageDTO.getQos());

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Message copied to form: {}", getConnectionId());
        }
    }

    @Override
    public Supplier<MessageListViewConfig> produceListViewConfig() {
        return () -> {
            ConnectionConfigDTO config = SettingsProvider.getInstance()
                    .getConnectionConfigs()
                    .stream()
                    .filter(c -> c.getId().equals(getConnectionId()))
                    .findFirst()
                    .orElse(ConnectionConfigDTO.builder().publishListViewConfig(new MessageListViewConfig()).build());

            return config.producePublishListViewConfig() != null ? config.producePublishListViewConfig() : new MessageListViewConfig();
        };

    }

    private void executeOnCopyMessageToFormExtensions(MessagePropertiesDTO messageDTO) {
        pluginSystem.getExtensions(MessageContextMenuHook.class)
                .forEach(p -> p.onCopyMessageToPublishForm(getConnectionId(), new MessageExtensionDTO(messageDTO)));
    }

    @SuppressWarnings("unused")
    public void onPublishSucceeded(@Subscribe PublishEvent event) {
        event.getMessageDTO().setPublishStatus(PublishStatus.SUCCEEDED);
        messageListViewController.onNewMessage(MessageTransformer.dtoToProps(event.getMessageDTO()));
    }

    private void onPublishFailed(MessageDTO messageDTO, Throwable exception) {
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

    @SuppressWarnings("unused")
    public void onImportStarted(@Subscribe ImportMessageStartedEvent event) {
        Platform.runLater(() -> {
            loadingViewController = LoadingViewController.showAsDialog(getConnectionId(),
                    resources.getString("publishViewControllerOpenFileTitle") + ": " + event.file().getAbsolutePath());
            loadingViewController.setProgress(1);
        });
    }

    @SuppressWarnings("unused")
    public void onImportSucceeded(@Subscribe ImportMessageSuccessEvent event) {
        Platform.runLater(() -> {
            topicComboBox.setValue(event.messageDTO().getTopic());
            retainedCheckBox.setSelected(event.messageDTO().isRetained());
            qosComboBox.setValue(event.messageDTO().getQos());
            payloadCodeArea.replaceText(event.messageDTO().getPayload());
            if (loadingViewController != null) {
                loadingViewController.close();
                loadingViewController = null;
            }
        });
    }

    @SuppressWarnings("unused")
    @Subscribe(ImportMessageFailedEvent.class)
    public void onImportFailed() {
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
        EventBus.fireAsync(new PublishListRemovedEvent(getConnectionId(), messageDTO));
    }

    @Override
    public void clearMessages() {
        EventBus.fireAsync(new PublishListClearEvent(getConnectionId()));
    }

    @Override
    public void setTabDirty() {
        delegate.setTabDirty();
    }

    public void cleanUp() {
        this.messageListViewController.cleanUp();
        EventBus.unregister(this);
    }
}
