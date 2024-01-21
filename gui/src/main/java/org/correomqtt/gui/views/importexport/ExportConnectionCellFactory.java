package org.correomqtt.gui.views.importexport;

import dagger.assisted.AssistedFactory;
import org.controlsfx.control.CheckListView;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;

@AssistedFactory
public interface ExportConnectionCellFactory {
    ExportConnectionCell create(CheckListView<ConnectionPropertiesDTO> listView);

}
