package com.correomqtt.plugin.ui.subscribe.topic

import com.intellij.ui.components.JBList
import com.intellij.ui.components.JBScrollPane
import org.correomqtt.core.pubsub.SubscribeEvent
import org.correomqtt.core.pubsub.UnsubscribeEvent
import org.correomqtt.di.DefaultBean
import org.correomqtt.di.Observes
import java.awt.BorderLayout
import java.awt.event.MouseAdapter
import java.awt.event.MouseEvent
import javax.swing.*

@DefaultBean
class TopicListView() : JPanel(BorderLayout()) {

    private val listModel = DefaultListModel<String>()
    private val jbList = JBList(listModel)

    init {
        jbList.selectionMode = ListSelectionModel.SINGLE_SELECTION
        jbList.cellRenderer = TopicListRenderer()
        jbList.addMouseListener(object : MouseAdapter() {
            override fun mouseClicked(e: MouseEvent) {
                if (e.clickCount == 2) {  // Double-click action
                    val selectedItem = jbList.selectedValue
                    if (selectedItem != null) {
                        JOptionPane.showMessageDialog(this@TopicListView, "You selected: $selectedItem")
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

    fun onSubscribeToTopic(@Observes event: SubscribeEvent) {
        println("Received subscription event for topic ${event.subscriptionDTO.topic}")
        listModel.addElement(event.subscriptionDTO.topic)
        jbList.revalidate();
        jbList.repaint();
    }

    fun onUnsubscribe(@Observes event: UnsubscribeEvent) {
        println("Received unsubscribe event for topic ${event.subscriptionDTO.topic}")
        listModel.removeElement(event.subscriptionDTO.topic);
    }
}