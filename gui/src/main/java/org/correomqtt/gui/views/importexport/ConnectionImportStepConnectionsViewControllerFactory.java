package org.correomqtt.gui.views.importexport;

import dagger.assisted.AssistedFactory;

@AssistedFactory
public interface ConnectionImportStepConnectionsViewControllerFactory {
    ConnectionImportStepConnectionsViewController create(ConnectionImportStepDelegate delegate);
}
