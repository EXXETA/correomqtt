package com.exxeta.correomqtt.business.services;

import com.exxeta.correomqtt.business.dispatcher.PublishDispatcher;
import com.exxeta.correomqtt.business.model.MessageDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class PublishService extends BaseService {

    private static final Logger LOGGER = LoggerFactory.getLogger(PublishService.class);

    private final MessageDTO messageDTO;

    public PublishService(String connectionId, MessageDTO messageDTO) {
        super(connectionId);
        this.messageDTO = messageDTO;
    }

    public void publish() {
        LOGGER.info(getConnectionMarker(), "Start publishing to topic: {}", messageDTO.getTopic());
        callSafeOnClient(client -> client.publish(messageDTO));
    }

    @Override
    public void onSucceeded() {
        LOGGER.info(getConnectionMarker(), "Publish to {} succeeded.", messageDTO.getTopic());
        PublishDispatcher.getInstance().onPublishSuceeded(connectionId, messageDTO);
    }

    @Override
    public void onCancelled() {
        LOGGER.info(getConnectionMarker(), "Publish to {} cancelled.", messageDTO.getTopic());
        PublishDispatcher.getInstance().onPublishCancelled(connectionId, messageDTO);
    }

    @Override
    public void onFailed(Throwable exception) {
        LOGGER.warn(getConnectionMarker(), "Publish to {} failed: ", messageDTO.getTopic(), exception);
        PublishDispatcher.getInstance().onPublishFailed(connectionId, messageDTO, exception);
    }

    @Override
    public void onRunning() {
        LOGGER.debug(getConnectionMarker(), "Publish to {} running", messageDTO.getTopic());
        PublishDispatcher.getInstance().onPublishRunning(connectionId, messageDTO);
    }

    @Override
    public void onScheduled() {
        LOGGER.debug(getConnectionMarker(), "Publish to {} scheduled", messageDTO.getTopic());
        PublishDispatcher.getInstance().onPublishScheduled(connectionId, messageDTO);
    }
}
