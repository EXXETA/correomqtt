package org.correomqtt.business.services;

import org.correomqtt.business.dispatcher.ConnectionLifecycleDispatcher;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class CleanUpService extends BaseService {

    private static final Logger LOGGER = LoggerFactory.getLogger(CleanUpService.class);
    public CleanUpService(String connectionId) {
        super(connectionId);
    }

    public void cleanUp() {
        LOGGER.info(getConnectionMarker(), "Start cleanup.");
        ConnectionLifecycleDispatcher.getInstance().onCleanUp(connectionId);
    }

    @Override
    public void onSucceeded() {

    }

    @Override
    public void onCancelled() {

    }

    @Override
    public void onFailed(Throwable exception) {

    }

    @Override
    public void onRunning() {

    }

    @Override
    public void onScheduled() {

    }
}
