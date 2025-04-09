package com.correomqtt.plugin.core.services.publish

import org.correomqtt.core.model.Qos

interface PublishService {
    /**
     * Publish a message on a specific topic for the active connection.
     */
    fun publish(topic: String, payload: String, qos: Qos, retained: Boolean)
}
