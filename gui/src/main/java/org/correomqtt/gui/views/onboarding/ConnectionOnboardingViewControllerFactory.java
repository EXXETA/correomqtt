package org.correomqtt.gui.views.onboarding;

import dagger.assisted.AssistedFactory;

@AssistedFactory
public interface ConnectionOnboardingViewControllerFactory {
    ConnectionOnboardingViewController create(ConnectionOnboardingDelegate connectionsOnboardingDelegate);
}
