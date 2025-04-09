package com.correomqtt.plugin.ui.main

import com.correomqtt.plugin.ui.common.components.DefaultPanel
import com.correomqtt.plugin.ui.common.events.ON_CONNECTION_SELECTED_TOPIC
import com.correomqtt.plugin.ui.common.events.ConnectionSelectionListener
import com.correomqtt.plugin.ui.connection.ConnectionTree
import com.correomqtt.plugin.ui.tab.TabManager
import com.intellij.openapi.project.Project
import com.intellij.ui.JBSplitter
import org.correomqtt.core.connection.ConnectionState
import org.correomqtt.core.connection.ConnectionStateChangedEvent
import org.correomqtt.core.model.Qos
import org.correomqtt.di.Assisted
import org.correomqtt.di.DefaultBean
import org.correomqtt.di.Inject
import org.correomqtt.di.Observes
import java.awt.BorderLayout
import javax.swing.JPanel

@DefaultBean
class ToolWindow @Inject constructor(@Assisted project: Project): JPanel(BorderLayout()) {
    private val splitter: JBSplitter = JBSplitter(false, 0.15f)
    private val tabManager: TabManager = com.correomqtt.plugin.ui.tab.TabManagerFactory().create(project)
    private val connectionTree: ConnectionTree = com.correomqtt.plugin.ui.connection.ConnectionTreeFactory().create(project)

    init {
        splitter.border = null;

        splitter.firstComponent = connectionTree
        splitter.secondComponent = DefaultPanel("No connection selected")

        add(splitter, BorderLayout.CENTER)
    }

    /**
     * Observes connection state changes and updates the tree item state accordingly.
     */
    fun onConnectionStateChanged(@Observes event: ConnectionStateChangedEvent) {
        println("Received connection state change event ${event.state}")

        if (event.state == ConnectionState.CONNECTED) {
            println("Connection selected: ${event.connectionId}")
            splitter.secondComponent = tabManager

            revalidate()
            repaint()
        }

    }
}