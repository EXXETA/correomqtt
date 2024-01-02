package org.correomqtt.business.fileprovider;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.eventbus.Subscribe;
import org.correomqtt.business.model.PublishHistoryListDTO;
import org.correomqtt.business.pubsub.PublishEvent;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class PersistPublishHistoryProvider extends BasePersistHistoryProvider<PublishHistoryListDTO> {

    private static final Logger LOGGER = LoggerFactory.getLogger(PersistPublishHistoryProvider.class);

    private static final String HISTORY_FILE_NAME = "publishHistory.json";
    private static final int MAX_ENTRIES = 100;

    private static final Map<String, PersistPublishHistoryProvider> instances = new HashMap<>();
    private static final Map<String, PublishHistoryListDTO> historyDTOs = new HashMap<>();

    private PersistPublishHistoryProvider(String id) {
        super(id);
        EventBus.register(this);
    }

    public static void activate(String id) {
        instances.computeIfAbsent(id, PersistPublishHistoryProvider::new);
    }

    public static synchronized PersistPublishHistoryProvider getInstance(String id) {
        instances.computeIfAbsent(id, PersistPublishHistoryProvider::new);
        return instances.get(id);
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
    Class<PublishHistoryListDTO> getDTOClass() {
        return PublishHistoryListDTO.class;
    }

    @Override
    void setDTO(String id, PublishHistoryListDTO dto) {
        historyDTOs.put(id, dto);
    }

    public List<String> getTopics(String connectionId) {
        return historyDTOs.get(connectionId).getTopics();
    }

    @SuppressWarnings("unused")
    public void onPublishSucceeded(@Subscribe PublishEvent event) {
        LOGGER.info("Persisting new publish history entry: {}", event.getMessageDTO().getTopic());

        List<String> topicsSet = getTopics(event.getConnectionId());
        String topic = event.getMessageDTO().getTopic();
        topicsSet.remove(topic);
        topicsSet.add(topic);
        while (topicsSet.size() > MAX_ENTRIES) {
            LOGGER.info("Removing last entry from publish history, cause limit of {} is reached.", MAX_ENTRIES);
            topicsSet.remove(topicsSet.iterator().next());
        }

        saveHistory(event.getConnectionId());
    }

    private void saveHistory(String connectionId) {
        try {
            new ObjectMapper().writeValue(getFile(), historyDTOs.get(connectionId));
        } catch (IOException e) {
            LOGGER.error("Failed to write " + getHistoryFileName(), e);
            EventBus.fireAsync(new PersistPublishHistoryWriteFailedEvent(e));
        }
    }

    @SuppressWarnings("unused")
    @Subscribe(ConnectionsUpdatedEvent.class)
    public void onConnectionsUpdated() {
        removeFileIfConnectionDeleted();
    }


    public void cleanUp() {
        EventBus.unregister(this);

        instances.remove(getConnectionId());
        historyDTOs.remove(getConnectionId());
    }
}


