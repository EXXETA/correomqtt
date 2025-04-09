package com.correomqtt.plugin.core.services.connection

import com.correomqtt.plugin.GuiCore
import com.correomqtt.plugin.core.services.subscribe.SubscribeService
import com.intellij.openapi.components.service
import com.intellij.openapi.project.Project
import org.correomqtt.core.fileprovider.PublishHistory
import org.correomqtt.core.fileprovider.PublishMessageHistory
import org.correomqtt.core.fileprovider.SubscriptionHistory
import org.correomqtt.core.model.*
import org.correomqtt.core.settings.SettingsManager
import org.correomqtt.di.SoyDi
import java.util.*

/**
 * Service which manages the MQTT connection lifecycle.
 */
class ConnectionManagerServiceImpl : ConnectionManagerService {

    private val guiCore: GuiCore = SoyDi.inject(GuiCore::class.java)

    private val settingsManager: SettingsManager = guiCore.getSettingsManager()

    /**
     * Holds all active connections to their list index.
     * If the user switches between connections, the connectionId is stored or retrieved from this map.
     */
    private val listIndexToConnectionId = mutableMapOf<Int, String>()

    private lateinit var activeConnectionId: String;

    override fun getConnections(): List<ConnectionConfigDTO> {
        return settingsManager.connectionConfigs
    }

    /**
     * Calls the correo backend connect task for the given connectionId.
     * If the connection was successful, the connectionId is set as the active connection.
     * @param connectionData The connection data.
     * @param index The index of the connection in the connection list.
     */
    override fun connect(connectionData: ConnectionConfigDTO, index: Int) {
        val connectionId = connectionData.id
        val name = connectionData.name

        if (listIndexToConnectionId.containsKey(index)) {
            println("Connection already established for the selected broker")
            setActiveConnectionId(listIndexToConnectionId[index]!!)
            return
        }

        guiCore.getConnectionLifecycleTaskFactory()
                .connectFactory
                .create(connectionId)
                .onSuccess {
                    listIndexToConnectionId[index] = connectionId
                    println("Successfully connected to $name, $connectionId");
                    setActiveConnectionId(connectionId)
                }
                .onError {
                    println("Error while connecting to $name, $connectionId")
                    setActiveConnectionId("")
                }
                .run()
    }

    /**
     * Switches the active connection to the connectionId when the user previously connected to it.
     * @param connectionId The connectionId to switch to.
     */
    override fun switch(connectionData: ConnectionConfigDTO): Boolean {
        val id = connectionData.id
        val name = connectionData.name

        if (listIndexToConnectionId.containsValue(id)) {
            println("Switching to $name, $id")
            setActiveConnectionId(id)

            return true;
        } else {
            println("Connection not established for $name, $id")
            return false;
        }
    }

    /**
     * Calls the correo backend disconnect task.
     * @param tabIndex The tabIndex of the connection to disconnect from.
     */
    override fun disconnect() {
        guiCore.getConnectionLifecycleTaskFactory()
                .disconnectFactory
                .create(this.activeConnectionId)
                .onSuccess {
                    setActiveConnectionId("")
                    println("Successfully disconnected for ${this.activeConnectionId}");
                }
                .onError {
                    println("Error while disconnecting from ${this.activeConnectionId}")
                }
                .run()
    }

    override fun setActiveConnectionId(connectionId: String) {
        println("Setting active connection to $connectionId")
        activeConnectionId = connectionId
    }

    override fun getActiveConnectionId(): String {
       return activeConnectionId
    }
}