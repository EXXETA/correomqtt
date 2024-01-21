package org.correomqtt.gui.views.scripting;

import dagger.assisted.AssistedFactory;

@AssistedFactory
public interface SingleExecutionViewControllerFactory {
    SingleExecutionViewController create(ExecutionPropertiesDTO executionPropertiesDTO);

}
