package com.correomqtt.plugin.ui.connection

import com.correomqtt.plugin.GuiCore
import com.correomqtt.plugin.core.services.connection.ConnectionManagerService
import com.correomqtt.plugin.core.services.history.HistoryManagerService
import com.intellij.icons.AllIcons
import com.intellij.openapi.components.service
import com.intellij.openapi.project.Project
import com.intellij.ui.ToolbarDecorator
import com.intellij.ui.components.JBLabel
import com.intellij.ui.treeStructure.Tree
import com.intellij.util.ui.JBUI
import com.intellij.util.ui.UIUtil
import org.correomqtt.core.connection.ConnectionState
import org.correomqtt.core.connection.ConnectionStateChangedEvent
import org.correomqtt.core.model.ConnectionConfigDTO
import org.correomqtt.di.Assisted
import org.correomqtt.di.DefaultBean
import org.correomqtt.di.Inject
import org.correomqtt.di.Observes
import java.awt.*
import java.awt.event.MouseAdapter
import java.awt.event.MouseEvent
import javax.swing.BorderFactory
import javax.swing.JPanel
import javax.swing.JTree
import javax.swing.tree.DefaultMutableTreeNode
import javax.swing.tree.DefaultTreeCellRenderer
import javax.swing.tree.DefaultTreeModel
import javax.swing.tree.TreePath

@DefaultBean
class ConnectionTree @Inject constructor(@Assisted project: Project, guiCore: GuiCore) : JPanel(BorderLayout()) {
    private val service = service<ConnectionManagerService>()
    private val historyService = service<HistoryManagerService>();
    private val settingsManager = guiCore.getSettingsManager()
    private val rootNode = DefaultMutableTreeNode("MQTT Connections")
    private val treeModel: DefaultTreeModel = DefaultTreeModel(rootNode)
    private val tree = Tree(treeModel).apply { isRootVisible = true }
    private val connectionStateMap = mutableMapOf<String, ConnectionState>()

    init {
        val connections = service.getConnections();

        connections.forEach { connectionConfig ->

            val connectionNode = DefaultMutableTreeNode(connectionConfig)
            rootNode.add(connectionNode)
            // Hier werden die spezifischen Verbindungen zu diesem Knoten hinzugefügt
            addConnectionNodes(connectionNode, connectionConfig)
        }

        if (rootNode.childCount > 0) {
            tree.expandPath(TreePath(rootNode.path))
        }


        val decorator = ToolbarDecorator.createDecorator(tree)
            .setAddAction { addNode(project) }
            .setRemoveAction { removeNode() }
            .setAddIcon(AllIcons.General.Add)

        val root = decorator.createPanel()
        root.border = BorderFactory.createMatteBorder(0, 0, 0, 1, UIUtil.getBoundsColor())

        add(root, BorderLayout.CENTER)

        // Custom renderer for tree nodes
        tree.cellRenderer = object : DefaultTreeCellRenderer() {
            override fun getTreeCellRendererComponent(
                tree: JTree?,
                value: Any?,
                sel: Boolean,
                expanded: Boolean,
                leaf: Boolean,
                row: Int,
                hasFocus: Boolean
            ): java.awt.Component {
                val component = super.getTreeCellRendererComponent(
                    tree, value, sel, expanded, leaf, row, hasFocus
                )
                openIcon = AllIcons.Webreferences.MessageQueue
                closedIcon = AllIcons.Webreferences.MessageQueue

                if (leaf && value is DefaultMutableTreeNode) {
                    val connectionInfo = value.userObject as? ConnectionConfigDTO
                    if (connectionInfo != null) {
                        // Create a custom panel to hold the text and the icon
                        // Create a custom panel to hold the icons and text
                        val panel = JPanel(GridBagLayout())
                        panel.isOpaque = false
                        val gbc = GridBagConstraints()

                        // Server icon
                        val serverIconLabel = JBLabel(AllIcons.Webreferences.Server)
                        serverIconLabel.preferredSize = Dimension(16, 16)
                        gbc.gridx = 0
                        gbc.gridy = 0
                        gbc.insets = JBUI.insetsRight(5) // Right padding
                        panel.add(serverIconLabel, gbc)

                        val connectionState = connectionStateMap[connectionInfo.id]

                        gbc.gridx = 1
                        gbc.insets = JBUI.insetsRight(5) // Right padding

                        when (connectionState) {
                            ConnectionState.CONNECTED -> {
                                val checkLabel = JBLabel(AllIcons.General.InspectionsOK)
                                checkLabel.preferredSize = Dimension(16, 16)
                                panel.add(checkLabel, gbc)
                            }

                            ConnectionState.DISCONNECTED_GRACEFUL, ConnectionState.DISCONNECTED_UNGRACEFUL, ConnectionState.DISCONNECTING -> {
                                val crossLabel = JBLabel(AllIcons.General.Error)
                                crossLabel.preferredSize = Dimension(16, 16)
                                panel.add(crossLabel, gbc)
                            }

                            ConnectionState.CONNECTING -> {
                                val loadingLabel = JBLabel(AllIcons.Actions.Refresh)
                                loadingLabel.preferredSize = Dimension(16, 16)
                                panel.add(loadingLabel, gbc)
                            }

                            ConnectionState.RECONNECTING -> {
                                val loadingLabel = JBLabel(AllIcons.Actions.Refresh)
                                loadingLabel.preferredSize = Dimension(16, 16)
                                panel.add(loadingLabel, gbc)
                            }

                            null -> {
                                val loadingLabel = JBLabel(AllIcons.General.Error)
                                loadingLabel.preferredSize = Dimension(16, 16)
                                panel.add(loadingLabel, gbc)
                            }
                        }

                        // Create a label for the text
                        val labelText = "${connectionInfo.name}  (${connectionInfo.hostAndPort})"
                        val textLabel = JBLabel(labelText)

                        // Add the text label to the panel
                        gbc.gridx = 2
                        gbc.weightx = 1.0
                        gbc.fill = GridBagConstraints.HORIZONTAL

                        // Add the label and the icon to the panel
                        panel.add(textLabel, gbc)

                        if (connectionState == ConnectionState.CONNECTED) {
                            textLabel.font = textLabel.font.deriveFont(Font.BOLD)
                        }

                        return panel
                    }
                }
                return component
            }
        }

        /**
         * Double click listener. Triggers a connection to the selected connection.
         */
        tree.addMouseListener(object : MouseAdapter() {
            override fun mouseClicked(e: MouseEvent) {
                if (e.clickCount == 2) {
                    val selectedNode = tree.lastSelectedPathComponent as? DefaultMutableTreeNode
                    if (selectedNode != null) {
                        val nodeData = selectedNode.userObject as? ConnectionConfigDTO
                        if (nodeData != null) {
                            // Hier wird die spezifische Verbindung verarbeitet
                            service.connect(nodeData, tree.getRowForPath(tree.selectionPath))
                            /* project.messageBus.syncPublisher(ON_CONNECTION_SELECTED_TOPIC).onConnectionSelected(
                                 nodeData.name, nodeData.id
                             )*/
                        }
                    }
                }

                if (e.clickCount == 1) {
                    val selectedNode = tree.lastSelectedPathComponent as? DefaultMutableTreeNode
                    if (selectedNode != null) {
                        val nodeData = selectedNode.userObject as? ConnectionConfigDTO
                        if (nodeData != null) {
                            service.switch(nodeData)
                            /* if (isConnected) {


                                 project.messageBus.syncPublisher(ON_CONNECTION_SELECTED_TOPIC).onConnectionSelected(
                                     nodeData.name, nodeData.id
                                 )
                             }*/
                        }
                    }
                }
            }
        })
    }

    /**
     * Observes connection state changes and updates the tree item state accordingly.
     */
    fun onConnectionStateChanged(@Observes event: ConnectionStateChangedEvent) {
        println("Received connection state change event ${event.state}")
        connectionStateMap[event.connectionId] = event.state
        tree.revalidate()
        tree.repaint()

        if (event.state == ConnectionState.CONNECTED) {
            val connectionId = event.connectionId
            historyService.setupHistory(event.connectionId)
            println("Connection established for $connectionId")
        }
    }

    private fun addConnectionNodes(parentNode: DefaultMutableTreeNode, connectionConfig: ConnectionConfigDTO) {
        // Hier kannst du spezifische Verbindungen hinzufügen
        val specificConnections = getConnectionDetails(connectionConfig) // Beispiel: Fiktive Methode

        specificConnections.forEach { specificConnection ->
            val specificConnectionNode = DefaultMutableTreeNode(specificConnection)
            parentNode.add(specificConnectionNode) // Füge den spezifischen Verbindungs-Knoten hinzu
        }

    }

    private fun getConnectionDetails(connectionConfig: ConnectionConfigDTO): List<ConnectionConfigDTO> {
        // Dummy-Methode zum Abrufen spezifischer Verbindungen (hier können Sie Ihre Logik hinzufügen)
        return listOf() // Gibt eine leere Liste zurück oder relevante Verbindungen
    }

    private fun addNode(project: Project) {
        val dialog = AddConnectionDialog(project)

        if (dialog.showAndGet()) {
            val connectionDTO = dialog.getConnectionDTO()
            val newNode = DefaultMutableTreeNode(connectionDTO)

            println("Received new node name: $connectionDTO")

            val connections = service.getConnections().toMutableList()
            connections.add(connectionDTO)

            settingsManager.saveConnections(connections, "CorreoMQTT_Plugin").run {}

            rootNode.add(newNode)
            treeModel.reload(rootNode)
        }
    }

    private fun removeNode() {
        // Remove the selected node
        val selectedPath = tree.selectionPath
        if (selectedPath != null) {
            val selectedNode = selectedPath.lastPathComponent as DefaultMutableTreeNode
            if (selectedNode != rootNode) { // Prevent removing the root node
                selectedNode.removeFromParent()
                treeModel.reload(rootNode)
            }
        }
    }
}


