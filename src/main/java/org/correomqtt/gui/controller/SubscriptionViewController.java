package org.correomqtt.gui.controller;

import org.correomqtt.business.dispatcher.ConnectionLifecycleDispatcher;
import org.correomqtt.business.dispatcher.ConnectionLifecycleObserver;
import org.correomqtt.business.dispatcher.PersistSubscriptionHistoryDispatcher;
import org.correomqtt.business.dispatcher.PersistSubscriptionHistoryObserver;
import org.correomqtt.business.dispatcher.ShortcutDispatcher;
import org.correomqtt.business.dispatcher.ShortcutObserver;
import org.correomqtt.business.dispatcher.SubscribeDispatcher;
import org.correomqtt.business.dispatcher.SubscribeObserver;
import org.correomqtt.business.dispatcher.UnsubscribeDispatcher;
import org.correomqtt.business.dispatcher.UnsubscribeObserver;
import org.correomqtt.business.exception.CorreoMqttException;
import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.model.Qos;
import org.correomqtt.business.model.SubscriptionDTO;
import org.correomqtt.business.services.PersistSubscriptionHistoryService;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.business.TaskFactory;
import org.correomqtt.gui.cell.QosCell;
import org.correomqtt.gui.cell.SubscriptionViewCell;
import org.correomqtt.gui.cell.TopicCell;
import org.correomqtt.gui.contextmenu.SubscriptionListMessageContextMenu;
import org.correomqtt.gui.contextmenu.SubscriptionListMessageContextMenuDelegate;
import org.correomqtt.gui.helper.AlertHelper;
import org.correomqtt.gui.helper.CheckTopicHelper;
import org.correomqtt.gui.model.MessagePropertiesDTO;
import org.correomqtt.gui.model.SubscriptionPropertiesDTO;
import org.correomqtt.gui.transformer.MessageTransformer;
import org.correomqtt.gui.transformer.SubscriptionTransformer;
import javafx.collections.FXCollections;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.ComboBox;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.AnchorPane;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.List;
import java.util.ResourceBundle;
import java.util.Set;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.stream.Collectors;

public class SubscriptionViewController extends BaseMessageBasedViewController implements
        SubscribeObserver,
        UnsubscribeObserver,
        ConnectionLifecycleObserver,
        ShortcutObserver,
        SubscriptionListMessageContextMenuDelegate,
        PersistSubscriptionHistoryObserver {

    private static final Logger LOGGER = LoggerFactory.getLogger(SubscriptionViewController.class);
    private static ResourceBundle resources;
    private final SubscriptionViewDelegate delegate;

    @FXML
    public AnchorPane subscribeBodyViewAnchor;

    @FXML
    public ComboBox<Qos> qosComboBox;

    @FXML
    public ComboBox<String> subscribeTopicComboBox;

    @FXML
    public ListView<SubscriptionPropertiesDTO> subscriptionListView;

    @FXML
    private Button unsubscribeButton;

    @FXML
    private Button unsubscribeAllButton;

    @FXML
    private Button selectAllButton;

    @FXML
    private Button selectNoneButton;
    private boolean afterSubscribe;

    public SubscriptionViewController(String connectionId, SubscriptionViewDelegate delegate) {
        super(connectionId);
        this.delegate = delegate;
        SubscribeDispatcher.getInstance().addObserver(this);
        UnsubscribeDispatcher.getInstance().addObserver(this);
        ConnectionLifecycleDispatcher.getInstance().addObserver(this);
        ShortcutDispatcher.getInstance().addObserver(this);
        PersistSubscriptionHistoryDispatcher.getInstance().addObserver(this);
    }

    static LoaderResult<SubscriptionViewController> load(String connectionId, SubscriptionViewDelegate delegate) {
        LoaderResult<SubscriptionViewController> result = load(SubscriptionViewController.class, "subscriptionView.fxml",
                () -> new SubscriptionViewController(connectionId, delegate));
        resources = result.getResourceBundle();
        return result;
    }

    @FXML
    public void initialize() {

        initMessageListView();

        qosComboBox.setItems(FXCollections.observableArrayList(Qos.values()));
        qosComboBox.getSelectionModel().selectFirst();
        qosComboBox.setCellFactory(QosCell::new);


        subscriptionListView.setItems(FXCollections.observableArrayList(SubscriptionPropertiesDTO.extractor()));
        subscriptionListView.setCellFactory(this::createCell);

        unsubscribeButton.setDisable(true);
        unsubscribeAllButton.setDisable(true);
        selectAllButton.setDisable(true);
        selectNoneButton.setDisable(true);

        subscribeTopicComboBox.getEditor().lengthProperty().addListener((observable, oldValue, newValue) -> {
            CheckTopicHelper.checkSubscribeTopic(subscribeTopicComboBox, false, afterSubscribe);
            if (newValue.toString().length() > 0) {
                afterSubscribe = false;
            }
        });

        initTopicComboBox();

    }

    private void initTopicComboBox() {
        List<String> topics = PersistSubscriptionHistoryService.getInstance(getConnectionId()).getTopics(getConnectionId());
        subscribeTopicComboBox.setItems(FXCollections.observableArrayList(topics));
        subscribeTopicComboBox.setCellFactory(TopicCell::new);

    }

    private ListCell<SubscriptionPropertiesDTO> createCell(ListView<SubscriptionPropertiesDTO> listView) {
        SubscriptionViewCell cell = new SubscriptionViewCell(listView);
        SubscriptionListMessageContextMenu contextMenu = new SubscriptionListMessageContextMenu(this);
        cell.setContextMenu(contextMenu);
        cell.itemProperty().addListener((observable, oldValue, newValue) -> {
            contextMenu.setObject(cell.getItem());
        });
       //cell.setOnMouseClicked(event -> onSubscriptionListClicked(event, cell.getItem()));
        cell.selectedProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue) {
                onSubscriptionSelected(cell.getItem());
            }
        });
        return cell;
    }

    private void subscribe() {
        if (!CheckTopicHelper.checkSubscribeTopic(subscribeTopicComboBox, true, afterSubscribe)) {
            return;
        }

        String topic = subscribeTopicComboBox.getValue();

        if (topic == null || topic.isEmpty()) {
            LOGGER.info("Topic must not be empty");
            subscribeTopicComboBox.getStyleClass().add("emptyError");
            return;
        }

        Qos selectedQos = qosComboBox.getSelectionModel().getSelectedItem();
        TaskFactory.subscribe(getConnectionId(), SubscriptionPropertiesDTO.builder()
                                                                          .topic(topic)
                                                                          .qos(selectedQos)
                                                                          .build());

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Subscribing to topic '{}': {}", topic, getConnectionId());
        }
    }

    private SubscriptionPropertiesDTO getSelectedSubscription() {
        return subscriptionListView.getSelectionModel().getSelectedItem();
    }

    @FXML
    private void onClickUnsubscribe() {

        SubscriptionPropertiesDTO selectedSubscription = getSelectedSubscription();

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Unsubscribe from topic '{}' clicked: {}", selectedSubscription.getTopic(), getConnectionId());
        }
        if (selectedSubscription != null) {
            unsubscribe(selectedSubscription);
        }
    }

    public void unsubscribe(SubscriptionPropertiesDTO subscriptionDTO) {
        TaskFactory.unsubscribe(getConnectionId(), subscriptionDTO);
    }

    @FXML
    public void selectAll() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Select all topics for filter clicked: {}", getConnectionId());
        }
        subscriptionListView.getItems().forEach(subscriptionDTO -> subscriptionDTO.setFiltered(true));
    }

    @Override
    public void filterOnly(SubscriptionPropertiesDTO dto) {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Filter only topic '{}': {}", dto.getTopic(), getConnectionId());
        }
        subscriptionListView.getItems().forEach(item -> item.setFiltered(dto.equals(item)));
    }

    @FXML
    public void selectNone() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Select none topic for filter clicked: {}", getConnectionId());
        }
        subscriptionListView.getItems().forEach(subscriptionDTO -> subscriptionDTO.setFiltered(false));
    }

    @FXML
    private void onClickUnsubscribeAll() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Unsubscribe from all topics clicked: {}", getConnectionId());
        }
        unsubscribeAll();
    }

    public void unsubscribeAll() {

        ConnectionHolder.getInstance()
                        .getConnection(getConnectionId())
                        .getClient()
                        .getSubscriptions()
                        .forEach(s -> TaskFactory.unsubscribe(getConnectionId(), SubscriptionTransformer.dtoToProps(s)));

        subscriptionListView.getItems().clear();

        unsubscribeButton.setDisable(true);
        unsubscribeAllButton.setDisable(true);
        selectAllButton.setDisable(true);
        selectNoneButton.setDisable(true);
    }


    public void onClickSubscribe(@SuppressWarnings("unused") ActionEvent actionEvent) {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Subscribe to topic clicked: {}", getConnectionId());
        }
        subscribe();
    }

    public void onClickSubscribeKey(KeyEvent actionEvent) {
        if (actionEvent.getCode() == KeyCode.ENTER) {
            subscribeTopicComboBox.setValue(subscribeTopicComboBox.getEditor().getText());
            if (subscribeTopicComboBox.getValue() == null) {
                return;
            }
            subscribe();
        }
    }

    @FXML
    public void onSubscriptionSelected(SubscriptionPropertiesDTO subscriptionDTO) {
        unsubscribeButton.setDisable(subscriptionDTO == null);
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Subscription selected '{}': {}", subscriptionDTO == null ? "N/A" : subscriptionDTO.getTopic(), getConnectionId());
        }
    }

    // TODO: if all existing subscriptions are not filtered and a new comes in, no new messages are shown in the list
    // only after reclick the checkbox it works

    @Override
    public void onMessageIncoming(MessageDTO messageDTO, SubscriptionDTO subscriptionDTO) {
        MessagePropertiesDTO messagePropertiesDTO = MessageTransformer.dtoToProps(messageDTO);
        messagePropertiesDTO.getSubscriptionDTOProperty().setValue(SubscriptionTransformer.dtoToProps(subscriptionDTO));
        messageListViewController.onNewMessage(messagePropertiesDTO);
    }

    @Override
    public void onSubscribedSucceeded(SubscriptionDTO subscriptionDTO) {
        afterSubscribe = true;
        subscribeTopicComboBox.getSelectionModel().select("");

        if (subscriptionDTO.isHidden()) {
            return;
        }

        SubscriptionPropertiesDTO subscriptionPropertiesDTO = SubscriptionTransformer.dtoToProps(subscriptionDTO);
        subscriptionListView.getItems().add(0, subscriptionPropertiesDTO);
        unsubscribeAllButton.setDisable(false);
        selectAllButton.setDisable(false); //TODO disable on demand
        selectNoneButton.setDisable(false); //TODO disable on demand

        subscriptionPropertiesDTO.getFilteredProperty().addListener((observable, oldValue, newValue) -> {
            if (oldValue != newValue) {
                updateFilter();
            }
        });
    }

    @Override
    public void onSubscribedCanceled(SubscriptionDTO subscriptionDTO) {

    }

    @Override
    public void onSubscribedFailed(SubscriptionDTO subscriptionDTO, Throwable exception) {
        String msg;
        if (exception instanceof CorreoMqttException) {
            msg = ((CorreoMqttException) exception).getInfo();
        } else {
            msg = "Exception in business layer: " + exception.getMessage();
        }
        AlertHelper.warn(resources.getString("subscribeViewControllerSubscriptionFailedTitle") + ": ", msg);
    }

    private void updateFilter() {

        Set<String> filteredTopics = subscriptionListView.getItems()
                                                         .stream()
                                                         .filter(dto -> dto.getFilteredProperty().getValue())
                                                         .map(dto -> dto.getTopicProperty().getValue())
                                                         .collect(Collectors.toSet());

        messageListViewController.setFilterPredicate(m -> {
            SubscriptionPropertiesDTO subscription = m.getSubscription();
            if (subscription == null) {
                return false;
            }
            return filteredTopics.contains(subscription.getTopic());

        });

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Filter updated", getConnectionId());
        }
    }

    @Override
    public void setUpToForm(MessagePropertiesDTO messageDTO) {
        delegate.setUpToForm(messageDTO);
    }

    @Override
    public void onDisconnectFromConnectionDeleted(String connectionId) {
        // do nothing
    }

    @Override
    public void onConnect() {
        // nothing to do
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
        subscriptionListView.getItems().clear();
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
    public void onSubscriptionShortcutPressed() {
        // nothing to do
    }

    @Override
    public void onClearIncomingShortcutPressed() {
        // nothing to do
    }

    @Override
    public void updateSubscriptions(String connectionId) {
        initTopicComboBox();
    }

    @Override
    public void errorReadingSubscriptionHistory(Throwable exception) {
// nothing to do
    }

    @Override
    public void errorWritingSubscriptionHistory(Throwable exception) {
// nothing to do
    }

    @Override
    public void onUnsubscribeSucceeded(SubscriptionDTO subscriptionDTO) {

        SubscriptionPropertiesDTO subscriptionToRemove = subscriptionListView.getItems().stream()
                                                                             .filter(s -> s.getTopic().equals(subscriptionDTO.getTopic()))
                                                                             .findFirst()
                                                                             .orElse(null);

        if (subscriptionToRemove != null) {
            subscriptionListView.getItems().remove(subscriptionToRemove);
            unsubscribeButton.setDisable(true);

            if (subscriptionListView.getItems().isEmpty()) {
                unsubscribeAllButton.setDisable(true);
                selectAllButton.setDisable(true);
                selectNoneButton.setDisable(true);
            }
        }
    }

    @Override
    public void onUnsubscribeCanceled(SubscriptionDTO subscriptionDTO) {
// nothing to do
    }

    @Override
    public void onUnsubscribeFailed(SubscriptionDTO subscriptionDTO, Throwable exception) {
// nothing to do
    }

    @Override
    public void removeMessage(MessageDTO messageDTO) {
        // nothing to do
    }

    @Override
    public void clearMessages() {
        // nothing to do
    }

    @Override
    public void setTabDirty() {
        delegate.setTabDirty();
    }
}

