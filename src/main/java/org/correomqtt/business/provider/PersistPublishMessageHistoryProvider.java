package org.correomqtt.business.provider;

import org.correomqtt.business.dispatcher.ConfigDispatcher;
import org.correomqtt.business.dispatcher.ConfigObserver;
import org.correomqtt.business.dispatcher.ConnectionLifecycleDispatcher;
import org.correomqtt.business.dispatcher.ConnectionLifecycleObserver;
import org.correomqtt.business.dispatcher.PersistPublishHistoryDispatcher;
import org.correomqtt.business.dispatcher.PublishGlobalDispatcher;
import org.correomqtt.business.dispatcher.PublishGlobalObserver;
import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.model.PublishMessageHistoryListDTO;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.atomic.AtomicInteger;

public class PersistPublishMessageHistoryProvider extends BasePersistHistoryProvider<PublishMessageHistoryListDTO>
        implements PublishGlobalObserver,
        ConnectionLifecycleObserver,
        ConfigObserver {

    private static final Logger LOGGER = LoggerFactory.getLogger(PersistPublishMessageHistoryProvider.class);

    private static final String HISTORY_FILE_NAME = "publishMessageHistory.json";
    private static final int MAX_ENTRIES = 100;

    private static final Map<String, PersistPublishMessageHistoryProvider> instances = new HashMap<>();
    private static final Map<String, PublishMessageHistoryListDTO> historyDTOs = new HashMap<>();


    private PersistPublishMessageHistoryProvider(String id) {
        super(id);
        PublishGlobalDispatcher.getInstance().addObserver(this);
        ConnectionLifecycleDispatcher.getInstance().addObserver(this);
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
        PersistPublishHistoryDispatcher.getInstance().errorReadingPublishHistory(e);
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

    @Override
    public void onPublishSucceeded(String connectionId, MessageDTO messageDTO) {
        LOGGER.info("Persisting new publish history entry: {}", messageDTO.getTopic());

        List<MessageDTO> messageList = getMessages(connectionId);
        messageList.add(0,messageDTO);
        while (messageList.size() > MAX_ENTRIES) {
            LOGGER.info("Removing last entry from publish history, cause limit of {} is reached.", MAX_ENTRIES);
            messageList.remove(messageList.size()-1);
        }
        saveHistory(connectionId);
    }

    private void saveHistory(String id) {
        try {
            new ObjectMapper().writeValue(getFile(), historyDTOs.get(id));
        } catch (IOException e) {
            LOGGER.error("Failed to write " + getHistoryFileName(), e);
            PersistPublishHistoryDispatcher.getInstance().errorWritingPublishHistory(e);
        }
    }

    @Override
    public void onPublishRemoved(String connectionId, MessageDTO messageDTO) {
        LOGGER.info("Removing {} from publish history for {}.", messageDTO.getTopic(), connectionId);
        List<MessageDTO> messageList = getMessages(connectionId);
        messageList.remove(messageDTO);
        saveHistory(connectionId);
    }

    @Override
    public void onPublishesCleared(String connectionId) {
        LOGGER.info("Clearing publish history for {}.", connectionId);
        List<MessageDTO> messageList = getMessages(connectionId);
        messageList.clear();
        saveHistory(connectionId);
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
}


