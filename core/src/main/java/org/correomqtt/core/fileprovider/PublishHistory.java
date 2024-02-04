package org.correomqtt.core.fileprovider;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.di.SoyEvents;
import org.correomqtt.di.Observes;
import org.correomqtt.core.model.PublishHistoryListDTO;
import org.correomqtt.core.pubsub.PublishEvent;
import org.correomqtt.core.settings.SettingsManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

@DefaultBean
public class PublishHistory extends BasePersistHistoryProvider<PublishHistoryListDTO> {

    private static final Logger LOGGER = LoggerFactory.getLogger(PublishHistory.class);

    private static final String HISTORY_FILE_NAME = "publishHistory.json";
    private static final int MAX_ENTRIES = 100;

    private static final Map<String, PublishHistory> instances = new HashMap<>();
    private static final Map<String, PublishHistoryListDTO> historyDTOs = new HashMap<>();

    @Inject
    public PublishHistory(SettingsManager settings,
                          SoyEvents soyEvents,
                          @Assisted String connectionId) {
        super(settings, soyEvents, connectionId);
    }

    @Override
    protected void readingError(Exception e) {
        soyEvents.fireAsync(new PersistPublishHistoryReadFailedEvent(e));
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
    public void onPublishSucceeded(@Observes PublishEvent event) {
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
            soyEvents.fireAsync(new PersistPublishHistoryWriteFailedEvent(e));
        }
    }

    @SuppressWarnings("unused")
    @Observes(ConnectionsUpdatedEvent.class)
    public void onConnectionsUpdated() {
        removeFileIfConnectionDeleted();
    }


    public void cleanUp() {
        instances.remove(getConnectionId());
        historyDTOs.remove(getConnectionId());
    }
}


