package org.correomqtt.gui.views.importexport;

import dagger.assisted.AssistedFactory;

@AssistedFactory
public interface ConnectionImportStepDecryptViewControllerFactory {
    ConnectionImportStepDecryptViewController create(ConnectionImportStepDelegate delegate);
}
