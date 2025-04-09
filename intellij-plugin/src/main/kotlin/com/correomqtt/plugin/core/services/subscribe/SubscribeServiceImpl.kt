package com.correomqtt.plugin.core.services.subscribe

import com.correomqtt.plugin.GuiCore
import com.correomqtt.plugin.core.services.connection.ConnectionManagerService
import com.intellij.openapi.components.service
import org.correomqtt.core.model.Qos
import org.correomqtt.core.model.SubscriptionDTO
import org.correomqtt.core.pubsub.SubscribeTaskFactory
import org.correomqtt.di.SoyDi

class SubscribeServiceImpl : SubscribeService {
    private val guiCore: GuiCore = SoyDi.inject(GuiCore::class.java)
    private val connectionService = service<ConnectionManagerService>()

    private val subscribeTaskFactory: SubscribeTaskFactory = guiCore.getSubscriptionManager()

    override fun subscribe(topic: String, qos: Qos) {
        val subscribeDTO = SubscriptionDTO.builder()
            .topic(topic)
            .qos(qos)
            .build()

        val activeConnectionId = connectionService.getActiveConnectionId()

        println("Try subscribing to $topic with connectionId $activeConnectionId")

        subscribeTaskFactory.create(activeConnectionId, subscribeDTO).onSuccess {
            println("Succesfully subscribed to $topic")
        }.run()
    }
}