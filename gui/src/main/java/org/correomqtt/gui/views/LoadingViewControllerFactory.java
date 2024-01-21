package org.correomqtt.gui.views;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;

@AssistedFactory
public interface LoadingViewControllerFactory {
    LoadingViewController create(@Assisted("connectionId") String connectionId,
                                 @Assisted("message") String message);

}
