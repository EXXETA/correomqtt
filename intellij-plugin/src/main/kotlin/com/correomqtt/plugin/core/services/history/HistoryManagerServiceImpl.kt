package com.correomqtt.plugin.core.services.history

import com.correomqtt.plugin.GuiCore
import com.correomqtt.plugin.core.services.subscribe.SubscribeService
import com.intellij.openapi.components.service
import org.correomqtt.core.fileprovider.HistoryManager
import org.correomqtt.core.fileprovider.PublishHistory
import org.correomqtt.core.fileprovider.PublishMessageHistory
import org.correomqtt.core.fileprovider.SubscriptionHistory
import org.correomqtt.core.model.Qos
import org.correomqtt.di.SoyDi

class HistoryManagerServiceImpl: HistoryManagerService {

    private val subscriptionService = service<SubscribeService>()

    private val guiCore: GuiCore = SoyDi.inject(GuiCore::class.java)

    private val historyManager: HistoryManager = guiCore.getHistoryManager()

    private lateinit var publishHistory: PublishHistory;

    private lateinit var publishMessageHistory: PublishMessageHistory;

    private lateinit var subscriptionHistory: SubscriptionHistory;

    override fun setupHistory(connectionId: String) {
        publishHistory = historyManager.activatePublishHistory(connectionId)
        subscriptionHistory = historyManager.activateSubscriptionHistory(connectionId)
        publishMessageHistory = historyManager.activatePublishMessageHistory(connectionId)

        subscriptionHistory.getTopics(connectionId).forEach {
            subscriptionService.subscribe(it, Qos.AT_MOST_ONCE)
        }
    }
}