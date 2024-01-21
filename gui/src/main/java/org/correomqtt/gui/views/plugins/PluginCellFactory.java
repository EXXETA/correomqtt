package org.correomqtt.gui.views.plugins;

import dagger.assisted.AssistedFactory;
import javafx.scene.control.ListView;
import org.correomqtt.gui.model.PluginInfoPropertiesDTO;

@AssistedFactory
public interface PluginCellFactory {
    PluginCell create(ListView<PluginInfoPropertiesDTO> listView);

}
