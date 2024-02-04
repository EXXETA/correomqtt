package org.correomqtt.core.fileprovider;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.eventbus.Subscribe;
import org.correomqtt.core.model.MessageDTO;
import org.correomqtt.core.model.PublishMessageHistoryListDTO;
import org.correomqtt.core.pubsub.PublishEvent;
import org.correomqtt.core.pubsub.PublishListClearEvent;
import org.correomqtt.core.pubsub.PublishListRemovedEvent;
import org.correomqtt.core.settings.SettingsManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

@DefaultBean
public class PublishMessageHistory extends BasePersistHistoryProvider<PublishMessageHistoryListDTO> {

    private static final Logger LOGGER = LoggerFactory.getLogger(PublishMessageHistory.class);

    private static final String HISTORY_FILE_NAME = "publishMessageHistory.json";
    private static final int MAX_ENTRIES = 100;

    private static final Map<String, PublishMessageHistoryListDTO> historyDTOs = new HashMap<>();

    @Inject
    PublishMessageHistory(SettingsManager settings,
                          EventBus eventBus,
                          @Assisted String connectionId) {
        super(settings, eventBus, connectionId);
        eventBus.register(this);
    }

    @Override
    protected void readingError(Exception e) {
        eventBus.fireAsync(new PersistPublishHistoryReadFailedEvent(e));
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

    public void onPublishSucceeded(@Subscribe PublishEvent event) {
        LOGGER.info("Persisting new publish history entry: {}", event.getMessageDTO().getTopic());

        List<MessageDTO> messageList = getMessages(event.getConnectionId());
        messageList.add(0, event.getMessageDTO());
        while (messageList.size() > MAX_ENTRIES) {
            LOGGER.info("Removing last entry from publish history, cause limit of {} is reached.", MAX_ENTRIES);
            messageList.remove(messageList.size() - 1);
        }
        saveHistory(event.getConnectionId());
    }

    private void saveHistory(String id) {
        try {
            new ObjectMapper().writeValue(getFile(), historyDTOs.get(id));
        } catch (IOException e) {
            LOGGER.error("Failed to write " + getHistoryFileName(), e);
            eventBus.fireAsync(new PersistPublishHistoryWriteFailedEvent(e));
        }
    }

    @SuppressWarnings("unused")
    public void onPublishRemoved(@Subscribe PublishListRemovedEvent event) {
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

    @SuppressWarnings("unused")
    @Subscribe(ConnectionsUpdatedEvent.class)
    public void onConnectionsUpdated() {
        removeFileIfConnectionDeleted();
    }

    public void cleanUp() {
        eventBus.unregister(this);

        historyDTOs.remove(getConnectionId());
    }
}


