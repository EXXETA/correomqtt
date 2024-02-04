package org.correomqtt.gui.views.connections;

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
import org.correomqtt.core.CoreManager;
import org.correomqtt.core.concurrent.SimpleTaskErrorResult;
import org.correomqtt.core.connection.ConnectionStateChangedEvent;
import org.correomqtt.di.SoyEvents;
import org.correomqtt.di.Observes;
import org.correomqtt.core.exception.CorreoMqttException;
import org.correomqtt.core.importexport.messages.ImportMessageFailedEvent;
import org.correomqtt.core.importexport.messages.ImportMessageStartedEvent;
import org.correomqtt.core.importexport.messages.ImportMessageSuccessEvent;
import org.correomqtt.core.importexport.messages.ImportMessageTaskFactory;
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
import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.gui.model.MessagePropertiesDTO;
import org.correomqtt.gui.plugin.spi.MessageContextMenuHook;
import org.correomqtt.gui.plugin.spi.PublishMenuHook;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.transformer.MessageTransformer;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.utils.AutoFormatPayload;
import org.correomqtt.gui.utils.CheckTopicHelper;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.LoadingViewController;
import org.correomqtt.gui.views.LoadingViewControllerFactory;
import org.correomqtt.gui.views.cell.QosCellFactory;
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

@DefaultBean
public class PublishViewController extends BaseMessageBasedViewController {

    private static final Logger LOGGER = LoggerFactory.getLogger(PublishViewController.class);
    private final PublishTaskFactory publishTaskFactory;
    private final QosCellFactory qosCellFactory;
    private final AutoFormatPayload autoFormatPayload;
    private final TopicCellFactory topicCellFactory;
    private final AlertHelper alertHelper;
    private final LoadingViewControllerFactory loadingViewControllerFactory;
    private final ImportMessageTaskFactory importMessageTaskFactory;
    private final SoyEvents soyEvents;
    private final PublishViewDelegate delegate;
    private ResourceBundle resources;
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



    @Inject
    public PublishViewController(CoreManager coreManager,
                                 PublishTaskFactory publishTaskFactory,
                                 QosCellFactory qosCellFactory,
                                 AutoFormatPayload autoFormatPayload,
                                 ThemeManager themeManager,
                                 MessageListViewControllerFactory messageListViewControllerFactory,
                                 TopicCellFactory topicCellFactory,
                                 AlertHelper alertHelper,
                                 LoadingViewControllerFactory loadingViewControllerFactory,
                                 ImportMessageTaskFactory importMessageTaskFactory,
                                 SoyEvents soyEvents,
                                 @Assisted String connectionId,
                                 @Assisted PublishViewDelegate delegate) {
        super(coreManager, themeManager, messageListViewControllerFactory, connectionId);
        this.publishTaskFactory = publishTaskFactory;
        this.qosCellFactory = qosCellFactory;
        this.autoFormatPayload = autoFormatPayload;
        this.topicCellFactory = topicCellFactory;
        this.alertHelper = alertHelper;
        this.loadingViewControllerFactory = loadingViewControllerFactory;
        this.importMessageTaskFactory = importMessageTaskFactory;
        this.soyEvents = soyEvents;
        this.delegate = delegate;
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
        qosComboBox.setCellFactory(qosCellFactory::create);

        coreManager.getPluginManager().getExtensions(PublishMenuHook.class).forEach(p -> {
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
        publishViewFormatToggleButton.setOnMouseClicked(mouseEvent -> autoFormatPayload.autoFormatPayload(
                payloadCodeArea.getText(),
                publishViewFormatToggleButton.isSelected(),
                getConnectionId(),
                payloadCodeArea,
                payloadCodeAreaChangeListener));

        payloadCodeArea.textProperty().addListener(payloadCodeAreaChangeListener);

        coreManager.getSettingsManager().getConnectionConfigs().stream()
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
        autoFormatPayload.autoFormatPayload(payloadCodeArea.getText(), publishViewFormatToggleButton.isSelected(), getConnectionId(), payloadCodeArea, payloadCodeAreaChangeListener);
    }

    private void initTopicComboBox() {
        List<String> topics = coreManager.getHistoryManager().activatePublishHistory(getConnectionId()).getTopics(getConnectionId());
        topicComboBox.setItems(FXCollections.observableArrayList(topics));
        topicComboBox.setCellFactory(topicCellFactory::create);
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

    private void onPublishFailed(SimpleTaskErrorResult result, MessageDTO messageDTO) {
        messageDTO.setPublishStatus(PublishStatus.FAILED);
        messageListViewController.onNewMessage(MessageTransformer.dtoToProps(messageDTO));

        String msg;
        if (result.getUnexpectedError() instanceof CorreoMqttException correoMqttException) {
            msg = correoMqttException.getInfo();
        } else {
            msg = "Exception in business layer: " + result.getUnexpectedError().getMessage();
        }
        alertHelper.warn(resources.getString("publishViewControllerPublishFailedTitle"),
                resources.getString("publishViewControllerPublishFailedContent") + ": " + messageDTO.getTopic() + ": " + msg);
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
            importMessageTaskFactory.create(file)
                    .onError(error -> alertHelper.unexpectedAlert(error.getUnexpectedError()))
                    .run();
        }
    }

    @SuppressWarnings("unused")
    public void onConnectionChangedEvent(@Observes ConnectionStateChangedEvent event) {
        if (event.getState() == CONNECTED) {
            // reverse order, because first message in history must be last one to add
            new LinkedList<>(coreManager.getHistoryManager().activatePublishMessageHistory(getConnectionId())
                    .getMessages(getConnectionId()))
                    .descendingIterator()
                    .forEachRemaining(messageDTO -> messageListViewController.onNewMessage(MessageTransformer.dtoToProps(messageDTO)));
        }
    }

    @SuppressWarnings("unused")
    public void onPublishSucceeded(@Observes PublishEvent event) {
        event.getMessageDTO().setPublishStatus(PublishStatus.SUCCEEDED);
        messageListViewController.onNewMessage(MessageTransformer.dtoToProps(event.getMessageDTO()));
    }

    @SuppressWarnings("unused")
    public void onImportStarted(@Observes ImportMessageStartedEvent event) {
        Platform.runLater(() -> {
            loadingViewController = loadingViewControllerFactory.create(getConnectionId(),
                            resources.getString("publishViewControllerOpenFileTitle") + ": " + event.file().getAbsolutePath())
                    .showAsDialog();
            loadingViewController.setProgress(1);
        });
    }

    @SuppressWarnings("unused")
    public void onImportSucceeded(@Observes ImportMessageSuccessEvent event) {
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
    @Observes(ImportMessageFailedEvent.class)
    public void onImportFailed() {
        Platform.runLater(() -> {
            if (loadingViewController != null) {
                loadingViewController.close();
                loadingViewController = null;
            }
            alertHelper.warn(resources.getString("publishViewControllerImportFileFailedTitle"),
                    resources.getString("publishViewControllerImportFileFailedContent"));
        });
    }

    @Override
    public void removeMessage(MessageDTO messageDTO) {
        soyEvents.fireAsync(new PublishListRemovedEvent(getConnectionId(), messageDTO));
    }

    @Override
    public void clearMessages() {
        soyEvents.fireAsync(new PublishListClearEvent(getConnectionId()));
    }

    @Override
    public void setTabDirty() {
        delegate.setTabDirty();
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
            ConnectionConfigDTO config = coreManager.getSettingsManager()
                    .getConnectionConfigs()
                    .stream()
                    .filter(c -> c.getId().equals(getConnectionId()))
                    .findFirst()
                    .orElse(ConnectionConfigDTO.builder().publishListViewConfig(new MessageListViewConfig()).build());

            return config.producePublishListViewConfig() != null ? config.producePublishListViewConfig() : new MessageListViewConfig();
        };

    }

    private void executeOnCopyMessageToFormExtensions(MessagePropertiesDTO messageDTO) {
        coreManager.getPluginManager().getExtensions(MessageContextMenuHook.class)
                .forEach(p -> p.onCopyMessageToPublishForm(getConnectionId(), MessageTransformer.propsToExtensionDTO(messageDTO)));
    }

    public void cleanUp() {
        this.messageListViewController.cleanUp();
    }
}
