package org.correomqtt.gui.views.cell;

import dagger.assisted.AssistedFactory;
import javafx.scene.control.ListView;
import org.correomqtt.core.model.GenericTranslatable;

@AssistedFactory
public interface GenericCellFactory<T extends GenericTranslatable> {
    GenericCell<T> create(ListView<T> listView);

}
