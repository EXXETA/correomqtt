package com.correomqtt.plugin.ui.common.events

import com.intellij.util.messages.Topic
import org.correomqtt.core.fileprovider.PublishHistory
import org.correomqtt.core.fileprovider.PublishMessageHistory
import org.correomqtt.core.fileprovider.SubscriptionHistory
import org.correomqtt.core.model.MessageDTO
import org.correomqtt.core.model.Qos

interface ConnectionSelectionListener {
    /**
     * User selects a connection from the connection tree.
     * @param name the name of the connection.
     * @param connectionId the id of the connection.
     */
    fun onConnectionSelected(
        name: String,
        connectionId: String,
    )
}

interface MessageSelectionListener {
    /**
     * User selects a message from the message list.
     * @param message the selected message information.
     */
    fun onMessageSelected(message: MessageDTO)
}

interface PublishListener {
    /**
     * User clicks the publish button.
     * @param topic the topic to publish.
     * @param qos quality of service of the message.
     * @param retained message will be retained on the broker.
     */
    fun onPublishListener(topic: String, qos: Qos, retained: Boolean)
}

val ON_CONNECTION_SELECTED_TOPIC = Topic.create("Connection selected", ConnectionSelectionListener::class.java)
val ON_MESSAGE_SELECTED = Topic.create("Message selected", MessageSelectionListener::class.java)
val ON_PUBLISH = Topic.create("Publish click", PublishListener::class.java)