package org.correomqtt.gui.views.connections;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedInject;
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
import org.correomqtt.core.concurrent.SimpleTaskErrorResult;
import org.correomqtt.core.connection.ConnectionStateChangedEvent;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.eventbus.Subscribe;
import org.correomqtt.core.exception.CorreoMqttException;
import org.correomqtt.core.fileprovider.PersistPublishHistoryProvider;
import org.correomqtt.core.fileprovider.PersistPublishMessageHistoryProvider;
import org.correomqtt.business.settings.SettingsProvider;
import org.correomqtt.core.importexport.messages.ImportMessageFailedEvent;
import org.correomqtt.core.importexport.messages.ImportMessageStartedEvent;
import org.correomqtt.core.importexport.messages.ImportMessageSuccessEvent;
import org.correomqtt.core.importexport.messages.ImportMessageTask;
import org.correomqtt.core.model.ConnectionConfigDTO;
import org.correomqtt.core.model.ControllerType;
import org.correomqtt.core.model.MessageDTO;
import org.correomqtt.core.model.MessageListViewConfig;
import org.correomqtt.core.model.MessageType;
import org.correomqtt.core.model.PublishStatus;
import org.correomqtt.core.model.Qos;
import org.correomqtt.core.pubsub.PublishEvent;
import org.correomqtt.core.pubsub.PublishListClearEvent;
import org.correomqtt.core.pubsub.PublishListRemovedEvent;
import org.correomqtt.core.pubsub.PublishTaskFactory;
import org.correomqtt.gui.utils.AutoFormatPayload;
import org.correomqtt.gui.model.MessagePropertiesDTO;
import org.correomqtt.gui.transformer.MessageTransformer;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.utils.CheckTopicHelper;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.LoadingViewController;
import org.correomqtt.gui.views.cell.QosCell;
import org.correomqtt.core.plugin.PluginManager;
import org.correomqtt.gui.plugin.spi.MessageContextMenuHook;
import org.correomqtt.gui.plugin.spi.PublishMenuHook;
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

import static org.correomqtt.core.connection.ConnectionState.CONNECTED;

public class PublishViewController extends BaseMessageBasedViewController {

    private static final Logger LOGGER = LoggerFactory.getLogger(PublishViewController.class);
    private static ResourceBundle resources;
    private final PublishTaskFactory publishTaskFactory;
    private final PluginManager pluginManager;
    private final PublishViewDelegate delegate;

    @FXML
    private AnchorPane publishViewAnchor;

    @FXML
    private ComboBox<Qos> qosComboBox;

    @FXML
    private ComboBox<String> topicComboBox;

    @FXML
    private HBox pluginControlBox;

    @FXML
    private CheckBox retainedCheckBox;

    @FXML
    private Button publishButton;

    @FXML
    private CodeArea payloadCodeArea;

    @FXML
    private Pane codeAreaScrollPane;

    @FXML
    private ToggleButton publishViewFormatToggleButton;

    private LoadingViewController loadingViewController;
    private ChangeListener<String> payloadCodeAreaChangeListener;

    @AssistedInject
    public PublishViewController(PublishTaskFactory publishTaskFactory,
                                 PluginManager pluginManager,
                                 @Assisted String connectionId,
                                 @Assisted PublishViewDelegate delegate) {
        super(connectionId);
        this.publishTaskFactory = publishTaskFactory;
        this.pluginManager = pluginManager;
        this.delegate = delegate;
        EventBus.register(this);
    }

    LoaderResult<PublishViewController> load() {
        LoaderResult<PublishViewController> result = load(PublishViewController.class, "publishView.fxml", () -> this);
        resources = result.getResourceBundle();
        return result;
    }

    @FXML
    private void initialize() {
        initMessageListView();

        qosComboBox.setItems(FXCollections.observableArrayList(Qos.values()));
        qosComboBox.getSelectionModel().selectFirst();
        qosComboBox.setCellFactory(QosCell::new);

        pluginManager.getExtensions(PublishMenuHook.class).forEach(p -> {
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
    private void onClickPublishKey(KeyEvent actionEvent) {
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

        publishTaskFactory.create(getConnectionId(), messageDTO)
                .onError(r -> this.onPublishFailed(r, messageDTO))
                .run();

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Publishing to topic: {}: {}", messageDTO.getTopic(), getConnectionId());
        }
    }

    @FXML
    private void onClickScan() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Open file button clicked: {}", getConnectionId());
        }

        Stage stage = new Stage();
        stage.setAlwaysOnTop(true);
        final FileChooser fileChooser = new FileChooser();
        fileChooser.setTitle(resources.getString("publishViewControllerOpenFileTitle"));
        File file = fileChooser.showOpenDialog(stage);

        if (file != null) {
            new ImportMessageTask(file)
                    .onError(error -> AlertHelper.unexpectedAlert(error.getUnexpectedError()))
                    .run();
        }
    }

    @SuppressWarnings("unused")
    public void onConnectionChangedEvent(@Subscribe ConnectionStateChangedEvent event) {
        if (event.getState() == CONNECTED) {
            // reverse order, because first message in history must be last one to add
            new LinkedList<>(PersistPublishMessageHistoryProvider.getInstance(getConnectionId())
                    .getMessages(getConnectionId()))
                    .descendingIterator()
                    .forEachRemaining(messageDTO -> messageListViewController.onNewMessage(MessageTransformer.dtoToProps(messageDTO)));
        }
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
        pluginManager.getExtensions(MessageContextMenuHook.class)
                .forEach(p -> p.onCopyMessageToPublishForm(getConnectionId(), MessageTransformer.propsToExtensionDTO(messageDTO)));
    }

    @SuppressWarnings("unused")
    public void onPublishSucceeded(@Subscribe PublishEvent event) {
        event.getMessageDTO().setPublishStatus(PublishStatus.SUCCEEDED);
        messageListViewController.onNewMessage(MessageTransformer.dtoToProps(event.getMessageDTO()));
    }

    private void onPublishFailed(SimpleTaskErrorResult result, MessageDTO messageDTO) {
        messageDTO.setPublishStatus(PublishStatus.FAILED);
        messageListViewController.onNewMessage(MessageTransformer.dtoToProps(messageDTO));

        String msg;
        if (result.getUnexpectedError() instanceof CorreoMqttException correoMqttException) {
            msg = correoMqttException.getInfo();
        } else {
            msg = "Exception in business layer: " + result.getUnexpectedError().getMessage();
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
