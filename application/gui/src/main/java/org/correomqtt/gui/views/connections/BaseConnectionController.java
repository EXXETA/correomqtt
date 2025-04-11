package org.correomqtt.gui.views.connections;

import lombok.Getter;
import org.correomqtt.core.CoreManager;
import org.correomqtt.di.ObservesFilter;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;
import org.slf4j.Marker;
import org.slf4j.MarkerFactory;

import static org.correomqtt.core.events.ObservesFilterNames.CONNECTION_ID;

public abstract class BaseConnectionController extends BaseControllerImpl {

    protected String connectionId;
    @Getter
    private String tabId;

    protected BaseConnectionController(CoreManager coreManager,
                                       ThemeManager themeManager,
                                       String connectionId) {
        super(coreManager, themeManager);
        this.connectionId = connectionId;
    }

    <C extends BaseControllerImpl> LoaderResult<C> load(final Class<C> controllerClazz,
                                                        final String fxml,
                                                        final String connectionId) {
        return load(controllerClazz,
                fxml,
                () -> controllerClazz.getDeclaredConstructor(String.class).newInstance(connectionId));
    }


    @ObservesFilter(CONNECTION_ID)
    public String getConnectionId() {
        return connectionId;
    }

    public void setConnectionId(String controllerUUID) {
        this.connectionId = controllerUUID;
    }

    Marker getConnectionMarker() {
        return MarkerFactory.getMarker(coreManager.getConnectionManager().getConfig(connectionId).getName());
    }

    public void setTabId(String tabId) {
        this.tabId = tabId;
    }
}
