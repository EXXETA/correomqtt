package org.correomqtt.gui.controller;

import org.correomqtt.business.eventbus.SubscribeFilter;
import org.correomqtt.business.utils.ConnectionHolder;
import org.slf4j.Marker;
import org.slf4j.MarkerFactory;

abstract class BaseConnectionController extends BaseControllerImpl {

    private String connectionId;
    private String tabId;

    BaseConnectionController(String connectionId) {
        super();
        this.connectionId = connectionId;
    }

    static <C extends BaseControllerImpl> LoaderResult<C> load(final Class<C> controllerClazz,
                                                               final String fxml,
                                                               final String connectionId) {
        return load(controllerClazz,
                    fxml,
                    () -> controllerClazz.getDeclaredConstructor(String.class).newInstance(connectionId));
    }


    @SubscribeFilter(value ="connectionId")
    public String getConnectionId() {
        return connectionId;
    }

    public void setConnectionId(String controllerUUID) {
        this.connectionId = controllerUUID;
    }

    Marker getConnectionMarker() {
        return MarkerFactory.getMarker(ConnectionHolder.getInstance().getConfig(connectionId).getName());
    }

    public String getTabId() {
        return tabId;
    }

    public void setTabId(String tabId) {
        this.tabId = tabId;
    }
}
