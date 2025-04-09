package com.correomqtt.plugin.core.services.subscribe

import org.correomqtt.core.model.Qos

interface SubscribeService {
    /**
     * Subscribe to a topic to the active connection.
     */
    fun subscribe(topic: String, qos: Qos)
}
