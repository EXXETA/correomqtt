package org.correomqtt.business.services;

import org.correomqtt.business.dispatcher.PublishDispatcher;
import org.correomqtt.business.exception.CorreoMqttExecutionException;
import org.correomqtt.business.model.MessageDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeoutException;

public class PublishService extends BaseService {

    private static final Logger LOGGER = LoggerFactory.getLogger(PublishService.class);

    private final MessageDTO messageDTO;

    public PublishService(String connectionId, MessageDTO messageDTO) {
        super(connectionId);
        this.messageDTO = messageDTO;
    }

    public void publish() {
        LOGGER.info(getConnectionMarker(), "Start publishing to topic: {}", messageDTO.getTopic());
        callSafeOnClient(client -> {
            try {
                client.publish(messageDTO);
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
                throw new CorreoMqttExecutionException(e);
            } catch (ExecutionException | TimeoutException e) {
                throw new CorreoMqttExecutionException(e);
            }
        });
    }

    @Override
    public void onSucceeded() {
        LOGGER.info(getConnectionMarker(), "Publish to {} succeeded.", messageDTO.getTopic());
        PublishDispatcher.getInstance().onPublishSucceeded(connectionId, messageDTO);
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
