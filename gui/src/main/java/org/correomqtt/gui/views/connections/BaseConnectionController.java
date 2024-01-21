package org.correomqtt.gui.views.connections;

import org.correomqtt.core.settings.SettingsProvider;
import org.correomqtt.core.eventbus.SubscribeFilter;
import org.correomqtt.core.utils.ConnectionHolder;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;
import org.slf4j.Marker;
import org.slf4j.MarkerFactory;

import static org.correomqtt.core.eventbus.SubscribeFilterNames.CONNECTION_ID;

public abstract class BaseConnectionController extends BaseControllerImpl {

    protected final ConnectionHolder connectionHolder;
    protected String connectionId;
    private String tabId;

    protected BaseConnectionController(SettingsProvider settingsProvider,
                                       ThemeManager themeManager,
                                       ConnectionHolder connectionHolder,
                                       String connectionId) {
        super(settingsProvider, themeManager);
        this.connectionHolder = connectionHolder;
        this.connectionId = connectionId;
    }

     <C extends BaseControllerImpl> LoaderResult<C> load(final Class<C> controllerClazz,
                                                               final String fxml,
                                                               final String connectionId) {
        return load(controllerClazz,
                fxml,
                () -> controllerClazz.getDeclaredConstructor(String.class).newInstance(connectionId));
    }


    @SubscribeFilter(CONNECTION_ID)
    public String getConnectionId() {
        return connectionId;
    }

    public void setConnectionId(String controllerUUID) {
        this.connectionId = controllerUUID;
    }

    Marker getConnectionMarker() {
        return MarkerFactory.getMarker(connectionHolder.getConfig(connectionId).getName());
    }

    public String getTabId() {
        return tabId;
    }

    public void setTabId(String tabId) {
        this.tabId = tabId;
    }
}
