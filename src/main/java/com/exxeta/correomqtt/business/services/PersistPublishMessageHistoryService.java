package com.exxeta.correomqtt.business.services;

import com.exxeta.correomqtt.business.dispatcher.ConfigDispatcher;
import com.exxeta.correomqtt.business.dispatcher.ConfigObserver;
import com.exxeta.correomqtt.business.dispatcher.ConnectionLifecycleDispatcher;
import com.exxeta.correomqtt.business.dispatcher.ConnectionLifecycleObserver;
import com.exxeta.correomqtt.business.dispatcher.PersistPublishHistoryDispatcher;
import com.exxeta.correomqtt.business.dispatcher.PublishGlobalDispatcher;
import com.exxeta.correomqtt.business.dispatcher.PublishGlobalObserver;
import com.exxeta.correomqtt.business.model.MessageDTO;
import com.exxeta.correomqtt.business.model.PublishMessageHistoryListDTO;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.util.HashMap;
import java.util.LinkedList;
import java.util.Map;
import java.util.concurrent.atomic.AtomicInteger;

public class PersistPublishMessageHistoryService extends BasePersistHistoryService<PublishMessageHistoryListDTO>
        implements PublishGlobalObserver,
        ConnectionLifecycleObserver,
        ConfigObserver {

    private static final Logger LOGGER = LoggerFactory.getLogger(PersistPublishMessageHistoryService.class);

    private static final String HISTORY_FILE_NAME = "publishMessageHistory.json";
    private static final int MAX_ENTRIES = 100;

    private static Map<String, PersistPublishMessageHistoryService> instances = new HashMap<>();
    private static Map<String, PublishMessageHistoryListDTO> historyDTOs = new HashMap<>();


    private PersistPublishMessageHistoryService(String id) {
        super(id);
        PublishGlobalDispatcher.getInstance().addObserver(this);
        ConnectionLifecycleDispatcher.getInstance().addObserver(this);
        ConfigDispatcher.getInstance().addObserver(this);
    }

    public static void activate(String id) {
        instances.computeIfAbsent(id, PersistPublishMessageHistoryService::new);
    }

    public static synchronized PersistPublishMessageHistoryService getInstance(String id) {
        return instances.computeIfAbsent(id, PersistPublishMessageHistoryService::new);
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

    public LinkedList<MessageDTO> getMessages(String connectionId) {
        return historyDTOs.get(connectionId).getMessages();
    }

    @Override
    public void onPublishSucceeded(String connectionId, MessageDTO messageDTO) {
        LOGGER.info("Persisting new publish history entry: {}", messageDTO.getTopic());

        LinkedList<MessageDTO> messageList = getMessages(connectionId);
        messageList.addFirst(messageDTO);
        while (messageList.size() > MAX_ENTRIES) {
            LOGGER.info("Removing last entry from publish history, cause limit of {} is reached.", MAX_ENTRIES);
            messageList.removeLast();
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
        LinkedList<MessageDTO> messageList = getMessages(connectionId);
        messageList.remove(messageDTO);
        saveHistory(connectionId);
    }

    @Override
    public void onPublishesCleared(String connectionId) {
        LOGGER.info("Clearing publish history for {}.", connectionId);
        LinkedList<MessageDTO> messageList = getMessages(connectionId);
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
    public void onSettingsUpdated() {
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


