package org.correomqtt.gui.controller;

import org.correomqtt.business.dispatcher.ConnectionLifecycleDispatcher;
import org.correomqtt.business.dispatcher.ConnectionLifecycleObserver;
import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.model.ControllerType;
import org.correomqtt.business.model.MessageType;
import org.correomqtt.business.model.PublishStatus;
import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.gui.cell.MessageViewCell;
import org.correomqtt.gui.contextmenu.MessageListContextMenu;
import org.correomqtt.gui.contextmenu.MessageListContextMenuDelegate;
import org.correomqtt.gui.model.MessagePropertiesDTO;
import org.correomqtt.gui.transformer.MessageTransformer;
import org.correomqtt.gui.utils.MessageUtils;
import org.correomqtt.plugin.manager.PluginManager;
import org.correomqtt.plugin.model.MessageExtensionDTO;
import org.correomqtt.plugin.spi.IncomingMessageHook;
import javafx.application.Platform;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.collections.transformation.FilteredList;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.scene.Node;
import javafx.scene.control.*;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.VBox;
import javafx.stage.Stage;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import lombok.Getter;

import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.function.Predicate;
import java.util.stream.Collectors;

public class MessageListViewController extends BaseConnectionController implements
        ConnectionLifecycleObserver,
        MessageListContextMenuDelegate,
        DetailViewDelegate ,
        MessageListViewDelegate{

    private static final Logger LOGGER = LoggerFactory.getLogger(MessageListViewController.class);

    private final MessageListViewDelegate delegate;

    @FXML
    Button clearMessagesButton;
    @FXML
    Button copyToFormButton;
    @FXML
    ListView<MessagePropertiesDTO> listView;
    @FXML
    Button showDetailsButton;
    @FXML
    protected SplitPane splitPane;
    @FXML
    private VBox messagesVBox;
    @FXML
    private TextField messageSearchTextField;
    @FXML
    private Button messageSearchClearButton;
    @FXML
    @Getter
    public ToggleButton FavoritesFilterButton;
    @FXML
    @Getter
    public ToggleButton FavoriteButton;
    @FXML
    protected ToggleButton showDetailViewButton;

    private ObservableList<MessagePropertiesDTO> messages;

    private FilteredList<MessagePropertiesDTO> filteredMessages;

    private FilteredList<MessagePropertiesDTO> favoritesMessages;

    private DetailViewController detailViewController;


    protected ControllerType controllerType = null;

    public MessageListViewController(String connectionId, MessageListViewDelegate delegate) {
        super(connectionId);
        ConnectionLifecycleDispatcher.getInstance().addObserver(this);
        this.delegate = delegate;
    }

    public static LoaderResult<MessageListViewController> load(String connectionId, MessageListViewDelegate delegate) {
        return load(MessageListViewController.class, "messageListView.fxml",
                () -> new MessageListViewController(connectionId, delegate));
    }

    @FXML
    public void initialize() {

        splitPane.getStyleClass().add(SettingsProvider.getInstance().getIconModeCssClass());

        copyToFormButton.setDisable(true);
        showDetailsButton.setDisable(true);
        clearMessagesButton.setDisable(true);
        FavoriteButton.setVisible(false);
        FavoritesFilterButton.setVisible(false);
        FavoriteButton.setDisable(true);

        messages = FXCollections.observableArrayList(MessagePropertiesDTO.extractor());

        filteredMessages = new FilteredList<>(messages, s -> true);

        favoritesMessages = new FilteredList<>(messages, MessagePropertiesDTO::isFavorited);


        listView.setItems(filteredMessages);
        listView.setCellFactory(this::createCell);
        splitPane.widthProperty().addListener((observable, oldValue, newValue) -> Platform.runLater(() -> calculateDetailView(newValue)));
        messageSearchTextField.textProperty().addListener((observable, oldValue, newValue) -> searchInMessages(newValue));
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

    @FXML
    private void resetMessageSearchTextField() {
        messageSearchTextField.textProperty().setValue("");

    }

    private ListCell<MessagePropertiesDTO> createCell(ListView<MessagePropertiesDTO> listView) {
        MessageViewCell cell = new MessageViewCell(listView);
        MessageListContextMenu contextMenu = new MessageListContextMenu(this);
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

    private void onCellClicked(MouseEvent event, MessagePropertiesDTO messageDTO) {
        if (messageDTO != null && event.getButton().equals(MouseButton.PRIMARY) && event.getClickCount() == 2) {
            DetailViewController.showAsDialog(messageDTO, getConnectionId(), this);
        }

        if(this.getSelectedMessage()!=null)
            FavoriteButton.setDisable(false);
            FavoriteButton.setSelected(this.getSelectedMessage().isFavorited());

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Message selected in list: {}: {}", (messageDTO != null) ? messageDTO.getTopic() : null, getConnectionId());
        }
    }

    @FXML
    public void clearList() {

        if (detailViewController != null) {
            detailViewController.setMessage(null);
        }

        List<MessagePropertiesDTO> messageList =messages.stream().filter(m-> !m.isFavorited()).collect(Collectors.toList());
        messages.removeAll(messageList);


        copyToFormButton.setDisable(true);
        showDetailsButton.setDisable(true);

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
        MessageUtils.saveMessage(getConnectionId(), messageDTO, stage);
    }

    @Override
    public void markMessageAsFavorite(MessagePropertiesDTO dto) {
        changeFavoriteStatus(dto);
    }


    void setFilterPredicate(Predicate<MessagePropertiesDTO> filterPredicate) {
        filteredMessages.setPredicate(filterPredicate);
    }


    Node getMainNode() {
        return splitPane;
    }

    private MessagePropertiesDTO getSelectedMessage() {
        return listView.getSelectionModel().getSelectedItem();
    }

    void onNewMessage(MessagePropertiesDTO messageDTO) {

        if (messageDTO.getSubscription() != null && messageDTO.getSubscription().isHidden()) {
            return;
        }

        if (messageDTO.getTopic().startsWith("$SYS")) {
            return;
        }

        delegate.setTabDirty();

        if (messageDTO.getPublishStatus() != null && messageDTO.getPublishStatus().equals(PublishStatus.PUBLISEHD)) {
            messages.stream()
                    .filter(m -> m.getMessageId().equals(messageDTO.getMessageId()))
                    .findFirst()
                    .ifPresentOrElse(m -> m.setPublishStatus(PublishStatus.PUBLISEHD), () -> addMessage(messageDTO));
            return;
        } else if (messageDTO.getPublishStatus() != null && messageDTO.getPublishStatus().equals(PublishStatus.SUCCEEDED)) {
            messages.stream()
                    .filter(m -> m.getMessageId().equals(messageDTO.getMessageId()))
                    .findFirst()
                    .ifPresentOrElse(m -> m.setPublishStatus(PublishStatus.SUCCEEDED), () -> addMessage(messageDTO));
            return;
        } else if (messageDTO.getPublishStatus() != null && messageDTO.getPublishStatus().equals(PublishStatus.FAILED)) {
            messages.stream()
                    .filter(m -> m.getMessageId().equals(messageDTO.getMessageId()))
                    .findFirst()
                    .ifPresentOrElse(m -> m.setPublishStatus(PublishStatus.FAILED), () -> addMessage(messageDTO));
            return;
        }

        if (messageDTO.getMessageType().equals(MessageType.INCOMING)) {
            addMessage(messageDTO);
        }
    }

    private void addMessage(MessagePropertiesDTO messageDTO) {
        final MessagePropertiesDTO updatedMessageDTO = executeOnMessageIncomingExtensions(messageDTO);
        Platform.runLater(() -> {
            messages.add(0, updatedMessageDTO);
            clearMessagesButton.setDisable(false);
        });
    }

    private MessagePropertiesDTO executeOnMessageIncomingExtensions(MessagePropertiesDTO messageDTO) {
        MessageExtensionDTO messageExtensionDTO = new MessageExtensionDTO(messageDTO);
        for (IncomingMessageHook p : PluginManager.getInstance().getExtensions(IncomingMessageHook.class)) {
            LOGGER.info("Incoming {}", p);
            messageExtensionDTO = p.onMessageIncoming(getConnectionId(), messageExtensionDTO);
        }
        return messageExtensionDTO.merge(messageDTO);
    }

    @FXML
    private void copyToForm() {
        delegate.setUpToForm(getSelectedMessage());
    }


    @FXML
    private void showDetailsOfMessage() {
        DetailViewController.showAsDialog(getSelectedMessage(), getConnectionId(), this);
    }

    @FXML
    private void toggleDetailView() {
        if (showDetailViewButton.isSelected()) {
            showDetailView();
        } else {
            closeDetailView();
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
            LoaderResult<DetailViewController> result = DetailViewController.load(getSelectedMessage(), getConnectionId(), this, true);
            detailViewController = result.getController();
            splitPane.getItems().add(result.getMainPane());

            SettingsProvider.getInstance().getConnectionConfigs().stream()
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
    @FXML
    void OnClickedFavoritesFilter() {
       if(FavoritesFilterButton.isSelected()) {
           listView.setItems(favoritesMessages);
       }
       else {
           listView.setItems(filteredMessages);
       }
    }
    @FXML
    void OnClickedChangeFavoriteStatus() {
        if (this.getSelectedMessage()!=null)
            changeFavoriteStatus(this.getSelectedMessage());
    }
    @Override
    public void changeFavoriteStatus(MessagePropertiesDTO messageDTO) {
        messageDTO.getIsFavoritedProperty().set(!messageDTO.isFavorited());
        delegate.changeFavoriteStatus(messageDTO);
    }

    @Override
    public void onDisconnectFromConnectionDeleted(String connectionId) {
        // do nothing
    }

    @Override
    public void onConnect() {
        setUpShortcuts();
    }


    private void setUpShortcuts() {
        listView.addEventHandler(KeyEvent.KEY_PRESSED, event -> {
            if (KeyCode.ENTER == event.getCode()) {
                showDetailsOfMessage();
            }
        });
    }

    @Override
    public void onConnectRunning() {
        // do nothing
    }

    @Override
    public void onConnectionFailed(Throwable message) {
        // do nothing
    }

    @Override
    public void onConnectionCanceled() {
        // do nothing
    }

    @Override
    public void onConnectionLost() {
        // do nothing
    }

    @Override
    public void onDisconnect() {
        // do nothing
    }

    @Override
    public void onConnectScheduled() {
        // do nothing
    }

    @Override
    public void onDisconnectCanceled() {
        // do nothing
    }

    @Override
    public void onDisconnectFailed(Throwable exception) {
        // do nothing
    }

    @Override
    public void onDisconnectRunning() {
        // do nothing
    }

    @Override
    public void onDisconnectScheduled() {
        // do nothing
    }

    @Override
    public void onConnectionReconnected() {
        // do nothing
    }

    @Override
    public void onReconnectFailed(AtomicInteger triedReconnects, int maxReconnects) {
        // do nothing
    }

    @Override
    public void showDetailsInSeparateWindow(MessagePropertiesDTO messageDTO) {
        // do nothing
    }

    @Override
    public void removeMessage(MessageDTO messageDTO) {
        // do nothing
    }

    @Override
    public void clearMessages() {
        // do nothing
    }

    @Override
    public void setTabDirty() {
        // do nothing
    }

    @Override
    public void setUpToForm(MessagePropertiesDTO messageDTO) {
        delegate.setUpToForm(messageDTO);
    }


}
