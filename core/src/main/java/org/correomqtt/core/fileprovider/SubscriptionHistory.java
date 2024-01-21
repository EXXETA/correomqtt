package org.correomqtt.core.fileprovider;

import com.fasterxml.jackson.databind.ObjectMapper;
import dagger.assisted.Assisted;
import dagger.assisted.AssistedInject;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.eventbus.Subscribe;
import org.correomqtt.core.model.SubscriptionDTO;
import org.correomqtt.core.model.SubscriptionHistoryListDTO;
import org.correomqtt.core.pubsub.SubscribeEvent;
import org.correomqtt.core.settings.SettingsManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class SubscriptionHistory extends BasePersistHistoryProvider<SubscriptionHistoryListDTO> {

    private static final Logger LOGGER = LoggerFactory.getLogger(SubscriptionHistory.class);

    private static final String HISTORY_FILE_NAME = "subscriptionHistory.json";

    private static final int MAX_ENTRIES = 100;

    private static Map<String, SubscriptionHistoryListDTO> historyDTOs = new HashMap<>();

    @AssistedInject
    SubscriptionHistory(SettingsManager settings, @Assisted String connectionId) {
        super(settings, connectionId);
        EventBus.register(this);
    }

    public void onSubscribedSucceeded(@Subscribe SubscribeEvent event) {

        SubscriptionDTO subscriptionDTO = event.getSubscriptionDTO();

        if (subscriptionDTO.isHidden()) {
            return;
        }

        LOGGER.info("Persisting new subscription history entry: {}", subscriptionDTO.getTopic());

        List<String> topicsSet = getTopics(getConnectionId());
        String topic = subscriptionDTO.getTopic();
        topicsSet.remove(topic);
        topicsSet.add(topic);
        while (topicsSet.size() > MAX_ENTRIES) {
            LOGGER.info("Removing last entry from subscription history, cause limit of {} is reached.", MAX_ENTRIES);
            topicsSet.remove(topicsSet.iterator().next());
        }

        saveHistory(getConnectionId()); //TODO is the connectionId not fixed?
    }

    public List<String> getTopics(String connectionId) {
        if (historyDTOs.get(connectionId) == null) {
            setDTO(connectionId, new SubscriptionHistoryListDTO(new ArrayList<>()));
        }
        return historyDTOs.get(connectionId).getTopics();
    }

    private void saveHistory(String connectionId) {
        try {
            new ObjectMapper().writeValue(getFile(), historyDTOs.get(connectionId));
            EventBus.fireAsync(new PersistSubscribeHistoryUpdateEvent(connectionId));
        } catch (IOException e) {
            LOGGER.error("Failed to write " + getHistoryFileName(), e);
            EventBus.fireAsync(new PersistSubscribeHistoryWriteFailedEvent(getConnectionId(), e));
        }
    }

    @Override
    String getHistoryFileName() {
        return HISTORY_FILE_NAME;
    }

    @Override
    Class<SubscriptionHistoryListDTO> getDTOClass() {
        return SubscriptionHistoryListDTO.class;
    }

    @Override
    void setDTO(String id, SubscriptionHistoryListDTO dto) {
        historyDTOs.put(id, dto);
    }

    @Override
    protected void readingError(Exception e) {
        EventBus.fireAsync(new PersistSubscribeHistoryReadFailedEvent(getConnectionId(), e));
    }

    @SuppressWarnings("unused")
    @Subscribe(ConnectionsUpdatedEvent.class)
    public void onConnectionsUpdated() {
        removeFileIfConnectionDeleted();
    }

    public void cleanUp() {
        EventBus.unregister(this);

        historyDTOs.remove(getConnectionId());
    }
}


