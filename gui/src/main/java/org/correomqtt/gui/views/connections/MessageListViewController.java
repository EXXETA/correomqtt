package org.correomqtt.gui.views.connections;

import javafx.application.Platform;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.collections.transformation.FilteredList;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.scene.Node;
import javafx.scene.control.Button;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.control.MenuButton;
import javafx.scene.control.SplitPane;
import javafx.scene.control.TextField;
import javafx.scene.control.ToggleButton;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.HBox;
import javafx.scene.layout.VBox;
import javafx.stage.Stage;
import org.controlsfx.control.textfield.TextFields;
import org.correomqtt.core.CoreManager;
import org.correomqtt.core.connection.ConnectionStateChangedEvent;
import org.correomqtt.core.model.ControllerType;
import org.correomqtt.core.model.LabelType;
import org.correomqtt.core.model.MessageListViewConfig;
import org.correomqtt.core.model.MessageType;
import org.correomqtt.core.model.PublishStatus;
import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.di.Observes;
import org.correomqtt.gui.contextmenu.MessageListContextMenu;
import org.correomqtt.gui.contextmenu.MessageListContextMenuDelegate;
import org.correomqtt.gui.contextmenu.MessageListContextMenuFactory;
import org.correomqtt.gui.controls.IconCheckMenuItem;
import org.correomqtt.gui.model.MessagePropertiesDTO;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.transformer.MessageTransformer;
import org.correomqtt.gui.utils.MessageUtils;
import org.correomqtt.gui.views.LoaderResult;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.function.Predicate;

import static org.correomqtt.core.connection.ConnectionState.CONNECTED;

@DefaultBean
public class MessageListViewController extends BaseConnectionController implements
        MessageListContextMenuDelegate,
        DetailViewDelegate {

    private static final Logger LOGGER = LoggerFactory.getLogger(MessageListViewController.class);

    private final MessageListViewDelegate delegate;
    private final DetailViewControllerFactory detailViewControllerFactory;
    private final MessageViewCellFactory messageViewCellFactory;
    private final MessageListContextMenuFactory messageListContextMenuFactory;
    private final MessageUtils messageUtils;
    @FXML
    protected SplitPane splitPane;
    @FXML
    protected ToggleButton showDetailViewButton;
    @FXML
    protected MenuButton showLabelsButton;
    @FXML
    protected IconCheckMenuItem changeDisplayRetained;
    @FXML
    protected IconCheckMenuItem changeDisplayQos;
    @FXML
    protected IconCheckMenuItem changeDisplayTimestamp;
    @FXML
    protected HBox messagesHBox;
    protected ControllerType controllerType = null;
    @FXML
    Button clearMessagesButton;
    @FXML
    Button copyToFormButton;
    @FXML
    ListView<MessagePropertiesDTO> listView;
    @FXML
    Button showDetailsButton;
    @FXML
    private VBox messagesVBox;
    private ObservableList<MessagePropertiesDTO> messages;
    private FilteredList<MessagePropertiesDTO> filteredMessages;
    private DetailViewController detailViewController;



    @Inject
    public MessageListViewController(CoreManager coreManager,
                                     DetailViewControllerFactory detailViewControllerFactory,
                                     ThemeManager themeManager,
                                     MessageViewCellFactory messageViewCellFactory,
                                     MessageListContextMenuFactory messageListContextMenuFactory,
                                     MessageUtils messageUtils,
                                     @Assisted String connectionId,
                                     @Assisted MessageListViewDelegate delegate) {
        super(coreManager, themeManager, connectionId);
        this.detailViewControllerFactory = detailViewControllerFactory;
        this.messageViewCellFactory = messageViewCellFactory;
        this.messageListContextMenuFactory = messageListContextMenuFactory;
        this.messageUtils = messageUtils;
        this.delegate = delegate;
    }

    public LoaderResult<MessageListViewController> load() {
        return load(MessageListViewController.class, "messageListView.fxml",
                () -> this);
    }

    @FXML
    private void initialize() {
        TextField messageSearchTextField;

        messageSearchTextField = TextFields.createClearableTextField();
        messageSearchTextField.setPromptText("Search topics ..."); //TODO translate
        messageSearchTextField.setId("messageSearchTextField");
        messageSearchTextField.getStyleClass().add("messageSearchTextField");
        messageSearchTextField.setMinHeight(27);

        messagesHBox.getChildren().add(3, messageSearchTextField);

        splitPane.getStyleClass().add(themeManager.getIconModeCssClass());

        copyToFormButton.setDisable(true);
        showDetailsButton.setDisable(true);
        clearMessagesButton.setDisable(true);

        MessageListViewConfig config = delegate.produceListViewConfig().get();

        if (config.isVisible(LabelType.QOS)) {
            changeDisplayQos.setSelected(true);
        }

        if (config.isVisible(LabelType.RETAINED)) {
            changeDisplayRetained.setSelected(true);
        }

        if (config.isVisible(LabelType.TIMESTAMP)) {
            changeDisplayTimestamp.setSelected(true);
        }

        messages = FXCollections.observableArrayList(MessagePropertiesDTO.extractor());
        filteredMessages = new FilteredList<>(messages, s -> true);

        listView.setItems(filteredMessages);
        listView.setCellFactory(this::createCell);

        splitPane.widthProperty().addListener((observable, oldValue, newValue) -> Platform.runLater(() -> calculateDetailView(newValue)));

        messageSearchTextField.textProperty().addListener((observable, oldValue, newValue) -> searchInMessages(newValue));
    }

    private ListCell<MessagePropertiesDTO> createCell(ListView<MessagePropertiesDTO> listView) {
        MessageViewCell cell = messageViewCellFactory.create(listView, this.delegate.produceListViewConfig());
        MessageListContextMenu contextMenu = messageListContextMenuFactory.create(this);
        cell.setContextMenu(contextMenu);
        cell.itemProperty().addListener((observable, oldValue, newValue) -> contextMenu.setObject(newValue));
        cell.setOnMouseClicked(event -> onCellClicked(event, cell.getItem()));
        cell.selectedProperty().addListener((observable, oldValue, newValue) -> {
            if (Boolean.TRUE.equals(newValue)) {
                copyToFormButton.setDisable(false);
                showDetailsButton.setDisable(false);
                if (detailViewController != null) {
                    detailViewController.setMessage(cell.getItem());
                }
            }
        });
        return cell;
    }

    public void calculateDetailView(Number newValue) {
        if (newValue.intValue() <= 670) {
            closeDetailView();
            showDetailViewButton.setDisable(true);
        } else {
            showDetailViewButton.setDisable(false);

            if (showDetailViewButton.isSelected()) {
                showDetailView();
            }
        }
    }

    private void searchInMessages(String newValue) {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Search for {} in messages: {}", newValue, getConnectionId());
        }

        filteredMessages.setPredicate(message -> {
            if (newValue == null || newValue.isEmpty()) {
                return true;
            }

            return message.getTopic().contains(newValue);
        });
    }

    private void onCellClicked(MouseEvent event, MessagePropertiesDTO messageDTO) {
        if (messageDTO != null && event.getButton().equals(MouseButton.PRIMARY) && event.getClickCount() == 2) {
            detailViewControllerFactory.create(messageDTO, getConnectionId(), this, false).showAsDialog();
        }

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Message selected in list: {}: {}", (messageDTO != null) ? messageDTO.getTopic() : null, getConnectionId());
        }
    }

    protected void closeDetailView() {
        if (this.detailViewController != null) {
            splitPane.getItems().remove(detailViewController.getMainNode());
            this.detailViewController = null;
        }
    }

    protected void showDetailView() {

        if (detailViewController == null) {
            LoaderResult<DetailViewController> result = detailViewControllerFactory.create(getSelectedMessage(), getConnectionId(), this, true).load();
            detailViewController = result.getController();
            splitPane.getItems().add(result.getMainRegion());

            coreManager.getSettingsManager().getConnectionConfigs().stream()
                    .filter(c -> c.getId().equals(getConnectionId()))
                    .findFirst()
                    .ifPresent(c -> {
                        if (!splitPane.getDividers().isEmpty()) {
                            if (controllerType == ControllerType.SUBSCRIBE) {
                                splitPane.getDividers().get(0).setPosition(c.getConnectionUISettings().getSubscribeDetailDividerPosition());
                            } else if (controllerType == ControllerType.PUBLISH) {
                                splitPane.getDividers().get(0).setPosition(c.getConnectionUISettings().getPublishDetailDividerPosition());
                            }
                        }
                    });
        }
    }

    private MessagePropertiesDTO getSelectedMessage() {
        return listView.getSelectionModel().getSelectedItem();
    }

    public double getDetailDividerPosition() {
        if (!splitPane.getDividers().isEmpty()) {
            return splitPane.getDividers().get(0).getPosition();
        } else {
            return 0.5;
        }
    }

    public boolean isDetailActive() {
        return showDetailViewButton.isSelected();
    }

    @Override
    @FXML
    public void clearList() {

        if (detailViewController != null) {
            detailViewController.setMessage(null);
        }

        messages.clear();

        copyToFormButton.setDisable(true);
        showDetailsButton.setDisable(true);
        clearMessagesButton.setDisable(true);

        delegate.clearMessages();

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Message list cleared: {}", getConnectionId());
        }
    }

    @Override
    public void removeMessage(MessagePropertiesDTO messageDTO) {
        messages.remove(messageDTO);
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Message removed from list: {}: {}", messageDTO.getTopic(), getConnectionId());
        }

        delegate.removeMessage(MessageTransformer.propsToDTO(messageDTO));
    }

    @Override
    public void saveMessage(MessagePropertiesDTO messageDTO) {
        Stage stage = (Stage) messagesVBox.getScene().getWindow();
        messageUtils.saveMessage(messageDTO, stage);
    }

    void setFilterPredicate(Predicate<MessagePropertiesDTO> filterPredicate) {
        filteredMessages.setPredicate(filterPredicate);
    }

    Node getMainNode() {
        return splitPane;
    }

    void onNewMessage(MessagePropertiesDTO messageDTO) {

        if (messageDTO.getSubscription() != null && messageDTO.getSubscription().isHidden()) {
            return;
        }

        if (messageDTO.getTopic().startsWith("$SYS")) {
            return;
        }

        delegate.setTabDirty();

        if (messageDTO.getPublishStatus() != null && messageDTO.getPublishStatus().equals(PublishStatus.PUBLISHED)) {
            messages.stream()
                    .filter(m -> m.getMessageId().equals(messageDTO.getMessageId()))
                    .findFirst()
                    .ifPresentOrElse(m -> {
                        m.update(messageDTO);
                        m.setPublishStatus(PublishStatus.PUBLISHED);
                    }, () -> addMessage(messageDTO));
            return;
        } else if (messageDTO.getPublishStatus() != null &&
                messageDTO.getPublishStatus().equals(PublishStatus.SUCCEEDED)) {
            messages.stream()
                    .filter(m -> m.getMessageId().equals(messageDTO.getMessageId()))
                    .findFirst()
                    .ifPresentOrElse(m -> {
                        m.update(messageDTO);
                        m.setPublishStatus(PublishStatus.SUCCEEDED);
                    }, () -> addMessage(messageDTO));
            return;
        } else if (messageDTO.getPublishStatus() != null &&
                messageDTO.getPublishStatus().equals(PublishStatus.FAILED)) {
            messages.stream()
                    .filter(m -> m.getMessageId().equals(messageDTO.getMessageId()))
                    .findFirst()
                    .ifPresentOrElse(m -> {
                        m.update(messageDTO);
                        m.setPublishStatus(PublishStatus.FAILED);
                    }, () -> addMessage(messageDTO));
            return;
        }

        if (messageDTO.getMessageType().equals(MessageType.INCOMING)) {
            addMessage(messageDTO);
        }
    }

    private void addMessage(MessagePropertiesDTO messageDTO) {
        Platform.runLater(() -> {
            messages.add(0, messageDTO);
            clearMessagesButton.setDisable(false);
        });
    }

    @FXML
    private void copyToForm() {
        delegate.setUpToForm(getSelectedMessage());
    }

    @FXML
    private void showLabelsInListView() {
        // nothing to do
    }

    @FXML
    private void toggleDetailView() {
        if (showDetailViewButton.isSelected()) {
            showDetailView();
        } else {
            closeDetailView();
        }
    }

    @FXML
    private void changeRetainDisplay(ActionEvent actionEvent) {
        IconCheckMenuItem checkMenuItem = (IconCheckMenuItem) actionEvent.getSource();
        setLabelVisibility(LabelType.RETAINED, checkMenuItem.isSelected());
    }

    private void setLabelVisibility(LabelType label, boolean visibility) {
        delegate.produceListViewConfig().get().setVisibility(label, visibility);
        coreManager.getSettingsManager().saveSettings();
        listView.refresh();

    }

    @FXML
    private void changeQosDisplay(ActionEvent actionEvent) {
        IconCheckMenuItem checkMenuItem = (IconCheckMenuItem) actionEvent.getSource();
        setLabelVisibility(LabelType.QOS, checkMenuItem.isSelected());
    }

    @FXML
    private void changeTimestampDisplay(ActionEvent actionEvent) {
        IconCheckMenuItem checkMenuItem = (IconCheckMenuItem) actionEvent.getSource();
        setLabelVisibility(LabelType.TIMESTAMP, checkMenuItem.isSelected());
    }

    @SuppressWarnings("unused")
    public void onConnectionChangedEvent(@Observes ConnectionStateChangedEvent event) {
        if (event.getState() == CONNECTED) {
            setUpShortcuts();
        }
    }

    private void setUpShortcuts() {
        listView.addEventHandler(KeyEvent.KEY_PRESSED, event -> {
            if (KeyCode.ENTER == event.getCode()) {
                showDetailsOfMessage();
            }
        });
    }

    @FXML
    private void showDetailsOfMessage() {
        detailViewControllerFactory.create(getSelectedMessage(), getConnectionId(), this, false).showAsDialog();
    }

    public void cleanUp() {
        if (this.detailViewController != null) {
            detailViewController.cleanUp();
        }
    }

    @Override
    public void showDetailsInSeparateWindow(MessagePropertiesDTO messageDTO) {
        // do nothing
    }

    @Override
    public void setUpToForm(MessagePropertiesDTO messageDTO) {
        delegate.setUpToForm(messageDTO);
    }
}
