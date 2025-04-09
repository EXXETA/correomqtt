package com.correomqtt.plugin.ui.publish

import com.correomqtt.plugin.core.services.publish.PublishService
import com.correomqtt.plugin.ui.common.events.ON_PUBLISH
import com.intellij.icons.AllIcons
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.DataContext
import com.intellij.openapi.components.service
import com.intellij.openapi.project.DumbAwareAction
import com.intellij.openapi.project.Project
import com.intellij.openapi.ui.ComboBox
import org.correomqtt.core.model.Qos
import java.awt.BorderLayout
import java.awt.Dimension
import java.awt.GridBagConstraints
import java.awt.GridBagLayout
import javax.swing.*

class Toolbar(project: Project) : JPanel(BorderLayout()) {
    private val textField = JTextField()
    private val comboBox = ComboBox(arrayOf(Qos.AT_MOST_ONCE, Qos.AT_LEAST_ONCE, Qos.EXACTLY_ONCE))
    private val publishButton = JButton("Publish")

    init {
        textField.preferredSize = Dimension(400, textField.preferredSize.height) // Set fixed width
        comboBox.preferredSize = Dimension(100, comboBox.preferredSize.height) // Set fixed width
        comboBox.maximumSize = Dimension(100, comboBox.preferredSize.height)
        publishButton.preferredSize = Dimension(100, publishButton.preferredSize.height)

        val publishAction = object : DumbAwareAction(
            "Publishing to ${textField.text}",
            "Subscribing action",
            AllIcons.Actions.Execute
        ) {
            override fun actionPerformed(e: AnActionEvent) {
                project.messageBus.syncPublisher(ON_PUBLISH).onPublishListener(
                    textField.text,
                    comboBox.selectedItem as Qos,
                    true
                )
            }
        }

        publishButton.addActionListener {
            publishAction.actionPerformed(
                AnActionEvent.createFromAnAction(
                    publishAction,
                    null,
                    "",
                    DataContext.EMPTY_CONTEXT
                )
            )
        }

        // Set the layout and constraints
        val gridBagLayout = GridBagLayout()
        val constraints = GridBagConstraints()
        val panel = JPanel(gridBagLayout)

        // Add the JTextField to the panel
        constraints.gridx = 0
        panel.add(textField, constraints)
        constraints.gridx = 1
        panel.add(comboBox, constraints)
        constraints.gridx = 2
        panel.add(publishButton, constraints)

        add(panel, BorderLayout.NORTH)
    }
}