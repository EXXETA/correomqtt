package com.correomqtt.plugin.ui.subscribe.payload

import com.correomqtt.plugin.ui.common.events.ON_MESSAGE_SELECTED
import com.correomqtt.plugin.ui.common.events.MessageSelectionListener
import com.correomqtt.plugin.ui.util.TimestampHelper
import com.intellij.openapi.project.Project
import com.intellij.ui.components.JBLabel
import com.intellij.util.ui.JBUI
import org.correomqtt.core.model.MessageDTO
import java.awt.BorderLayout
import java.awt.GridBagConstraints
import java.awt.GridBagLayout
import javax.swing.Box
import javax.swing.JPanel

/**
 * TODO:
 *  - Extract components
 *  - Correct grey tone for info text
 *  - Spacing
 */
class PayloadToolbar(project: Project) : JPanel(BorderLayout()) {
    init {
        val timeStampLabel = JBLabel("Timestamp: -")
        val qosLabel = JBLabel("QoS: -")
        val retainLabel = JBLabel("Retain: -")

        val container = JPanel(GridBagLayout())
        add(container, BorderLayout.WEST)

        val constraints = GridBagConstraints()
        constraints.insets = JBUI.insets(8, 0, 24, 0)
        constraints.gridx = 0
        constraints.weightx = 0.3
        constraints.weighty = 1.0

        container.add(timeStampLabel, constraints)

        constraints.gridx = 1
        container.add(Box.createHorizontalStrut(16), constraints)

        constraints.gridx = 2
        constraints.weightx = 0.3
        container.add(qosLabel, constraints)

        constraints.gridx = 3
        container.add(Box.createHorizontalStrut(16), constraints)

        constraints.gridx = 4
        constraints.weightx = 0.3
        container.add(retainLabel, constraints)

        val messageBus = project.messageBus.connect()

        messageBus.subscribe(ON_MESSAGE_SELECTED, object : MessageSelectionListener {
            override fun onMessageSelected(message: MessageDTO) {
                timeStampLabel.text = "Timestamp: ${TimestampHelper.format(message.dateTime.toString())}"
                qosLabel.text = "QoS: ${message.qos}"
                retainLabel.text = "Retain: ${message.isRetained}"
            }
        })
    }
}