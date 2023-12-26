package org.correomqtt.business.provider;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.business.dispatcher.ConfigDispatcher;
import org.correomqtt.business.dispatcher.ConfigObserver;
import org.correomqtt.business.dispatcher.ConnectionLifecycleDispatcher;
import org.correomqtt.business.dispatcher.ConnectionLifecycleObserver;
import org.correomqtt.business.dispatcher.PersistPublishHistoryDispatcher;
import org.correomqtt.business.dispatcher.PublishGlobalDispatcher;
import org.correomqtt.business.dispatcher.PublishGlobalObserver;
import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.model.PublishHistoryListDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.atomic.AtomicInteger;

public class PersistPublishHistoryProvider extends BasePersistHistoryProvider<PublishHistoryListDTO>
        implements PublishGlobalObserver,
        ConnectionLifecycleObserver,
        ConfigObserver {

    private static final Logger LOGGER = LoggerFactory.getLogger(PersistPublishHistoryProvider.class);

    private static final String HISTORY_FILE_NAME = "publishHistory.json";
    private static final int MAX_ENTRIES = 100;

    private static Map<String, PersistPublishHistoryProvider> instances = new HashMap<>();
    private static Map<String, PublishHistoryListDTO> historyDTOs = new HashMap<>();

    private PersistPublishHistoryProvider(String id) {
        super(id);
        PublishGlobalDispatcher.getInstance().addObserver(this);
        ConnectionLifecycleDispatcher.getInstance().addObserver(this);
        ConfigDispatcher.getInstance().addObserver(this);
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
        PersistPublishHistoryDispatcher.getInstance().errorReadingPublishHistory(e);
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

    @Override
    public void onPublishSucceeded(String connectionId, MessageDTO messageDTO) {
        LOGGER.info("Persisting new publish history entry: {}", messageDTO.getTopic());

        List<String> topicsSet = getTopics(connectionId);
        String topic = messageDTO.getTopic();
        topicsSet.remove(topic);
        topicsSet.add(topic);
        while (topicsSet.size() > MAX_ENTRIES) {
            LOGGER.info("Removing last entry from publish history, cause limit of {} is reached.", MAX_ENTRIES);
            topicsSet.remove(topicsSet.iterator().next());
        }

        saveHistory(connectionId);
    }

    private void saveHistory(String connectionId) {
        try {
            new ObjectMapper().writeValue(getFile(), historyDTOs.get(connectionId));
            PersistPublishHistoryDispatcher.getInstance().updatedPublishes(connectionId);
        } catch (IOException e) {
            LOGGER.error("Failed to write " + getHistoryFileName(), e);
            PersistPublishHistoryDispatcher.getInstance().errorWritingPublishHistory(e);
        }
    }

    @Override
    public void onPublishRemoved(String connectionId, MessageDTO messageDTO) {
        // nothing to do
    }

    @Override
    public void onPublishesCleared(String connectionId) {
        // nothing to do
    }

    @Override
    public void onDisconnectFromConnectionDeleted(String connectionId) {
        // nothing to do
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
    public void onConnectionLost() {
        // nothing to do
    }

    @Override
    public void onDisconnect() {

        PublishGlobalDispatcher.getInstance().removeObserver(this);
        ConnectionLifecycleDispatcher.getInstance().removeObserver(this);
        ConfigDispatcher.getInstance().removeObserver(this);

        instances.remove(getConnectionId());
        historyDTOs.remove(getConnectionId());
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
}


