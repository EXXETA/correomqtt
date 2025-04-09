package com.correomqtt.plugin.ui.subscribe.topic

import com.correomqtt.plugin.ui.common.components.Row
import com.intellij.icons.AllIcons
import com.intellij.ui.components.JBLabel
import java.awt.BorderLayout
import java.awt.Dimension
import java.awt.FlowLayout
import javax.accessibility.AccessibleContext
import javax.swing.JButton
import javax.swing.JPanel

//TODO Add qos to topic item
class TopicItem(topic: String, isSelected: Boolean) : Row(isSelected) {

    init {
        val unsubscribeButton = JButton(AllIcons.Actions.Close).apply {
            isBorderPainted = false
            isContentAreaFilled = false
            toolTipText = "Unsubscribe"
            addActionListener { } // Call unsubscribe function
            preferredSize = Dimension(16, 16)
        }

        // Create a container panel for the WEST side to hold both placeholderLabel and payload
        val eastPanel = JPanel(FlowLayout(FlowLayout.RIGHT, 16, 0)) // Adds horizontal gap of 16px between components
        eastPanel.isOpaque = false // Make sure the panel is transparent to show background

        // Ensure the westPanel takes up the full height of the Row, for vertical centering
        eastPanel.minimumSize = Dimension(0, 48)
        eastPanel.maximumSize = Dimension(Int.MAX_VALUE, 48)

        // Add the placeholder label and payload label to the west panel
        eastPanel.add(JBLabel("Qos 0"))
        eastPanel.add(unsubscribeButton)

        add(JBLabel(topic), BorderLayout.WEST)
        add(eastPanel, BorderLayout.EAST)
    }

    override fun getAccessibleContext(): AccessibleContext {
        return super.getAccessibleContext()
    }
}