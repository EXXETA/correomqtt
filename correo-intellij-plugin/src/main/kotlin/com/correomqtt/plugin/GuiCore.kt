package com.correomqtt.plugin

import org.correomqtt.core.CoreManager
import org.correomqtt.core.connection.ConnectionLifecycleTaskFactories
import org.correomqtt.core.fileprovider.HistoryManager
import org.correomqtt.core.pubsub.PublishTaskFactory
import org.correomqtt.core.pubsub.SubscribeTaskFactory
import org.correomqtt.core.settings.SettingsManager
import org.correomqtt.core.utils.ConnectionManager
import org.correomqtt.di.Inject
import org.correomqtt.di.SingletonBean

/**
 * GuiCore collects all relevant Correo backend services
 * and exposes them to the plugin components / services.
 */
@SingletonBean
class GuiCore @Inject constructor(
    private var coreManager: CoreManager,
    private var connectionLifecycleTaskFactories: ConnectionLifecycleTaskFactories,
    private var publishTaskFactory: PublishTaskFactory,
    private var subscribeTaskFactory: SubscribeTaskFactory,
) {

    private var connectionManager: ConnectionManager = coreManager.connectionManager
    private var settingsManager: SettingsManager = coreManager.settingsManager
    private var historyManager: HistoryManager = coreManager.historyManager

    fun getConnectionManager(): ConnectionManager {
        return this.connectionManager;
    }

    fun getSettingsManager(): SettingsManager {
        return this.settingsManager;
    }

    fun getHistoryManager(): HistoryManager {
        return this.historyManager;
    }

    fun getSubscriptionManager(): SubscribeTaskFactory {
        return this.subscribeTaskFactory;
    }

    fun getPublishTaskManager(): PublishTaskFactory {
        return this.publishTaskFactory;
    }

    fun getConnectionLifecycleTaskFactory(): ConnectionLifecycleTaskFactories {
        return this.connectionLifecycleTaskFactories;
    }
}