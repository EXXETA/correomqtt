package com.correomqtt.plugin.core.services.history

interface HistoryManagerService {

    /**
     * Setup all relevant history objects for publish, subscribe, publishMessage.
     */
    fun setupHistory(connectionId: String)
}