package org.correomqtt.business.fileprovider;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.business.dispatcher.ConfigDispatcher;
import org.correomqtt.business.dispatcher.ConfigObserver;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.eventbus.Subscribe;
import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.model.PublishMessageHistoryListDTO;
import org.correomqtt.business.pubsub.PublishEvent;
import org.correomqtt.business.pubsub.PublishListRemovedEvent;
import org.correomqtt.business.pubsub.PublishListClearEvent;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class PersistPublishMessageHistoryProvider extends BasePersistHistoryProvider<PublishMessageHistoryListDTO>
        implements
        ConfigObserver {

    private static final Logger LOGGER = LoggerFactory.getLogger(PersistPublishMessageHistoryProvider.class);

    private static final String HISTORY_FILE_NAME = "publishMessageHistory.json";
    private static final int MAX_ENTRIES = 100;

    private static final Map<String, PersistPublishMessageHistoryProvider> instances = new HashMap<>();
    private static final Map<String, PublishMessageHistoryListDTO> historyDTOs = new HashMap<>();


    private PersistPublishMessageHistoryProvider(String id) {
        super(id);
        EventBus.register(this);
        ConfigDispatcher.getInstance().addObserver(this);
    }

    public static void activate(String id) {
        instances.computeIfAbsent(id, PersistPublishMessageHistoryProvider::new);
    }

    public static synchronized PersistPublishMessageHistoryProvider getInstance(String id) {
        return instances.computeIfAbsent(id, PersistPublishMessageHistoryProvider::new);
    }

    @Override
    protected void readingError(Exception e) {
        EventBus.fireAsync(new PersistPublishHistoryReadFailedEvent(e));
    }

    @Override
    String getHistoryFileName() {
        return HISTORY_FILE_NAME;
    }

    @Override
    Class<PublishMessageHistoryListDTO> getDTOClass() {
        return PublishMessageHistoryListDTO.class;
    }

    @Override
    void setDTO(String id, PublishMessageHistoryListDTO dto) {
        historyDTOs.put(id, dto);
    }

    public List<MessageDTO> getMessages(String connectionId) {
        return historyDTOs.get(connectionId).getMessages();
    }

    public void onPublishSucceeded(@Subscribe PublishEvent event ) {
        LOGGER.info("Persisting new publish history entry: {}", event.getMessageDTO().getTopic());

        List<MessageDTO> messageList = getMessages(event.getConnectionId());
        messageList.add(0,event.getMessageDTO());
        while (messageList.size() > MAX_ENTRIES) {
            LOGGER.info("Removing last entry from publish history, cause limit of {} is reached.", MAX_ENTRIES);
            messageList.remove(messageList.size()-1);
        }
        saveHistory(event.getConnectionId());
    }

    private void saveHistory(String id) {
        try {
            new ObjectMapper().writeValue(getFile(), historyDTOs.get(id));
        } catch (IOException e) {
            LOGGER.error("Failed to write " + getHistoryFileName(), e);
            EventBus.fireAsync(new PersistPublishHistoryWriteFailedEvent(e));
        }
    }

    @SuppressWarnings("unused")
    public void onPublishRemoved(@Subscribe PublishListRemovedEvent event ) {
        LOGGER.info("Removing {} from publish history for {}.", event.getMessageDTO().getTopic(), event.getConnectionId());
        List<MessageDTO> messageList = getMessages(event.getConnectionId());
        messageList.remove(event.getMessageDTO());
        saveHistory(event.getConnectionId());
    }

    @SuppressWarnings("unused")
    public void onPublishesCleared(@Subscribe PublishListClearEvent event) {
        LOGGER.info("Clearing publish history for {}.", event.getConnectionId());
        List<MessageDTO> messageList = getMessages(event.getConnectionId());
        messageList.clear();
        saveHistory(event.getConnectionId());
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
        // nothing to do
    }

    @Override
    public void onConnectionsUpdated() {
        removeFileIfConnectionDeleted();
    }

    @Override
    public void onConfigPrepareFailed() {
        // nothing to do
    }

    public void cleanUp() {
        ConfigDispatcher.getInstance().removeObserver(this);
        EventBus.unregister(this);

        instances.remove(getConnectionId());
        historyDTOs.remove(getConnectionId());
    }
}


