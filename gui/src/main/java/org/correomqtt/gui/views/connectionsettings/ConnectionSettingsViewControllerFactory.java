package org.correomqtt.gui.views.connectionsettings;

import dagger.assisted.AssistedFactory;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;

@AssistedFactory
public interface ConnectionSettingsViewControllerFactory {
    ConnectionSettingsViewController create(ConnectionPropertiesDTO preSelecteded);
}
