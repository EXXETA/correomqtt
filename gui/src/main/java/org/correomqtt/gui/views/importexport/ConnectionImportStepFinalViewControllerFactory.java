package org.correomqtt.gui.views.importexport;

import dagger.assisted.AssistedFactory;

@AssistedFactory
public interface ConnectionImportStepFinalViewControllerFactory {
    ConnectionImportStepFinalViewController create(ConnectionImportStepDelegate delegate);
}
