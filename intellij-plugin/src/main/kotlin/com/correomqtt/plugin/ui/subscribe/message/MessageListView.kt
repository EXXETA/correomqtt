package com.correomqtt.plugin.ui.subscribe.message

import com.correomqtt.plugin.GuiCore
import com.correomqtt.plugin.ui.common.events.ConnectionSelectionListener
import com.correomqtt.plugin.ui.common.events.ON_CONNECTION_SELECTED_TOPIC
import com.correomqtt.plugin.ui.common.events.ON_MESSAGE_SELECTED
import com.intellij.openapi.project.Project
import com.intellij.ui.components.JBList
import com.intellij.ui.components.JBScrollPane
import org.correomqtt.core.fileprovider.PersistSubscribeHistoryUpdateEvent
import org.correomqtt.core.fileprovider.PublishHistory
import org.correomqtt.core.fileprovider.PublishMessageHistory
import org.correomqtt.core.fileprovider.SubscriptionHistory
import org.correomqtt.core.pubsub.IncomingMessageEvent
import org.correomqtt.di.Assisted
import org.correomqtt.di.DefaultBean
import org.correomqtt.di.Observes
import java.awt.BorderLayout
import java.awt.event.MouseAdapter
import java.awt.event.MouseEvent
import javax.swing.*

//TODO: Add message listener click to select message and display in message detail view
@DefaultBean
class MessageListView constructor(@Assisted project: Project) : JPanel(BorderLayout()) {
    private val listModel = DefaultListModel<IncomingMessageEvent>()
    private val jbList = JBList(listModel)

    init {
        jbList.selectionMode = ListSelectionModel.SINGLE_SELECTION

        jbList.cellRenderer = ListCellRenderer<IncomingMessageEvent> { _, value, _, isSelected, _ ->
            isOpaque = true;
            MessageItem(value, isSelected)
        }

        jbList.addMouseListener(object : MouseAdapter() {
            override fun mouseClicked(e: MouseEvent) {
                if (e.clickCount == 1) {
                    val selectedItem = jbList.selectedValue
                    if (selectedItem != null) {
                        println("Selected item: $selectedItem")
                        project.messageBus.syncPublisher(ON_MESSAGE_SELECTED).onMessageSelected(selectedItem.messageDTO)
                    }
                }

                if (e.clickCount == 2) {  // Double-click action
                    val selectedItem = jbList.selectedValue
                    if (selectedItem != null) {
                        JOptionPane.showMessageDialog(this@MessageListView, "You selected: $selectedItem")
                    }
                }
            }
        })

        // Wrap JBList in a scroll pane
        jbList.border = BorderFactory.createEmptyBorder()
        val scrollPane = JBScrollPane(jbList).apply {
            border = BorderFactory.createEmptyBorder()
        }
        add(scrollPane, BorderLayout.CENTER)
    }

    fun incomingMessageEvent(@Observes event: IncomingMessageEvent) {
        println("Received incoming message event for topic ${event.messageDTO.topic}")
        listModel.addElement(event)
        jbList.revalidate()
        jbList.repaint()
    }
}