package com.correomqtt.plugin.ui.subscribe

import com.correomqtt.plugin.core.services.subscribe.SubscribeService
import com.intellij.icons.AllIcons
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.DataContext
import com.intellij.openapi.components.service
import com.intellij.openapi.project.DumbAwareAction
import com.intellij.openapi.ui.ComboBox
import org.correomqtt.core.model.Qos
import java.awt.BorderLayout
import java.awt.Dimension
import java.awt.GridBagConstraints
import java.awt.GridBagLayout
import java.awt.event.KeyAdapter
import java.awt.event.KeyEvent
import javax.swing.JButton
import javax.swing.JPanel
import javax.swing.JTextField

class Toolbar: JPanel(BorderLayout()) {
    private val subscribeService = service<SubscribeService>()
    private val subscribeButton = JButton("Subscribe")
    private val textField = JTextField()
    private val comboBox = ComboBox(arrayOf(Qos.AT_MOST_ONCE, Qos.AT_LEAST_ONCE, Qos.EXACTLY_ONCE))

    init {
        // Subscribe Section
        textField.preferredSize = Dimension(400, textField.preferredSize.height) // Set fixed width
        comboBox.preferredSize = Dimension(100, comboBox.preferredSize.height) // Set fixed width
        comboBox.maximumSize = Dimension(100, comboBox.preferredSize.height) // Set fixed width
        subscribeButton.preferredSize = Dimension(100, subscribeButton.preferredSize.height) // Set fixed width


        val subscribeAction = object : DumbAwareAction(
            "Subscribing to ${textField.text}",
            "Subscribing action",
            AllIcons.Actions.Execute
        ) {
            override fun actionPerformed(e: AnActionEvent) {
                val topic = textField.text
                subscribeService.subscribe(topic, comboBox.selectedItem as Qos)
            }
        }
        subscribeButton.preferredSize = Dimension(100, subscribeButton.preferredSize.height) // Set fixed width
        subscribeButton.addActionListener {
            subscribeAction.actionPerformed(
                AnActionEvent.createFromAnAction(
                    subscribeAction,
                    null,
                    "",
                    DataContext.EMPTY_CONTEXT
                )
            )
        }

        subscribeButton.addKeyListener(object : KeyAdapter() {
            override fun keyPressed(e: KeyEvent) {
                if (e.keyCode == KeyEvent.VK_ENTER) {
                    subscribeAction.actionPerformed(
                        AnActionEvent.createFromAnAction(
                            subscribeAction,
                            null,
                            "",
                            DataContext.EMPTY_CONTEXT
                        )
                    )
                }
            }
        })

        // Add KeyListener to textField to detect Enter key press
        textField.addKeyListener(object : KeyAdapter() {
            override fun keyPressed(e: KeyEvent) {
                if (e.keyCode == KeyEvent.VK_ENTER) {
                    subscribeAction.actionPerformed(
                        AnActionEvent.createFromAnAction(
                            subscribeAction,
                            null,
                            "",
                            DataContext.EMPTY_CONTEXT
                        )
                    )
                }
            }
        })

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
        panel.add(subscribeButton, constraints)

        add(panel, BorderLayout.NORTH)

    }
}