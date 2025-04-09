package com.correomqtt.plugin.ui.subscribe.message

import com.correomqtt.plugin.ui.common.components.Row
import com.correomqtt.plugin.ui.util.IconHelper
import com.correomqtt.plugin.ui.util.TimestampHelper
import com.intellij.icons.AllIcons
import com.intellij.ui.components.JBLabel
import org.correomqtt.core.pubsub.IncomingMessageEvent
import java.awt.BorderLayout
import java.awt.Dimension
import java.awt.FlowLayout
import java.awt.Font
import javax.accessibility.AccessibleContext
import javax.swing.JPanel

class MessageItem(message: IncomingMessageEvent, isSelected: Boolean) : Row(isSelected) {
    init {
        val topic = JBLabel(message.messageDTO.topic)

        topic.font = topic.font.deriveFont(Font.BOLD)
        val timestamp = JBLabel(TimestampHelper.format(message.messageDTO.dateTime.toString()))
        val payload = JBLabel(message.messageDTO.payload)
        payload.maximumSize = Dimension(100, payload.height)

        val retainedIcon = if (message.messageDTO.isRetained) {
            AllIcons.Actions.Install
        } else {
            IconHelper.createGreyIcon(AllIcons.Actions.Install)
        }
        val placeholderLabel = JBLabel(retainedIcon)

        // Create a container panel for the WEST side to hold both placeholderLabel and payload
        val westPanel = JPanel(FlowLayout(FlowLayout.LEFT, 16, 0)) // Adds horizontal gap of 16px between components
        westPanel.isOpaque = false // Make sure the panel is transparent to show background

        // Add the placeholder label and payload label to the west panel
        westPanel.add(placeholderLabel)
        westPanel.add(payload)

        add(westPanel, BorderLayout.WEST)
        add(timestamp, BorderLayout.EAST)
    }

    override fun getAccessibleContext(): AccessibleContext {
        return super.getAccessibleContext()
    }
}