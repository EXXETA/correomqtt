package com.correomqtt.plugin.ui.publish

import com.correomqtt.plugin.core.services.publish.PublishService
import com.correomqtt.plugin.ui.common.components.PayloadArea
import com.correomqtt.plugin.ui.common.events.ON_PUBLISH
import com.correomqtt.plugin.ui.common.events.PublishListener
import com.intellij.openapi.components.service
import com.intellij.openapi.project.Project
import com.intellij.ui.components.JBScrollPane
import com.intellij.util.ui.JBUI
import org.correomqtt.core.model.Qos
import java.awt.BorderLayout
import java.awt.Dimension
import javax.swing.JPanel
import javax.swing.JScrollPane

/**
 * TODO:
 *  - Add Retained flag to publish
 */
class PublishView(project: Project) : JPanel(BorderLayout()) {
    private val publishService = service<PublishService>()
    private val payloadArea = PayloadArea().createJsonTextArea()
    private val rowScrollPane = JBScrollPane(payloadArea)

    init {
        project.messageBus.connect().subscribe(ON_PUBLISH, object : PublishListener {
            override fun onPublishListener(topic: String, qos: Qos, retained: Boolean) {
                publishService.publish(topic, payloadArea.text, qos, retained)
            }
        });

        rowScrollPane.verticalScrollBarPolicy = JScrollPane.VERTICAL_SCROLLBAR_AS_NEEDED
        rowScrollPane.horizontalScrollBarPolicy = JScrollPane.HORIZONTAL_SCROLLBAR_AS_NEEDED
        rowScrollPane.preferredSize = Dimension(width, height)
        rowScrollPane.border = JBUI.Borders.empty(16, 0)

        val wrapperPanel = JPanel(BorderLayout()).apply {
            border = JBUI.Borders.empty(16, 0)
            add(Toolbar(project), BorderLayout.WEST)
        }

        add(wrapperPanel, BorderLayout.NORTH)
        add(rowScrollPane, BorderLayout.CENTER)
    }
}