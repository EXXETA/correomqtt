package com.correomqtt.plugin.core.services.connection

import org.correomqtt.core.model.ConnectionConfigDTO

interface ConnectionManagerService {

    /**
     * Returns all broker connections.
     */
    fun getConnections(): List<ConnectionConfigDTO>

    /**
     * Connects to specific broker for the given connection configuration.
     */
    fun connect(connectionData: ConnectionConfigDTO, index: Int)

    /**
     * Disconnects from the active connection.
     */
    fun disconnect()

    /**
     * Set active connection.
     */
    fun setActiveConnectionId(connectionId: String)

    /**
     * Returns active connection.
     */
    fun getActiveConnectionId(): String

    /**
     * Switch to an already established connection.
     * Returns false if no connection is found.
     */
    fun switch(connectionData: ConnectionConfigDTO): Boolean
}