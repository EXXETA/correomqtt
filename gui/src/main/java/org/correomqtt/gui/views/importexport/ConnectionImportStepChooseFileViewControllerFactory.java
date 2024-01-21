package org.correomqtt.gui.views.importexport;

import dagger.assisted.AssistedFactory;

@AssistedFactory
public interface ConnectionImportStepChooseFileViewControllerFactory {
    ConnectionImportStepChooseFileViewController create(ConnectionImportStepDelegate delegate);
}
