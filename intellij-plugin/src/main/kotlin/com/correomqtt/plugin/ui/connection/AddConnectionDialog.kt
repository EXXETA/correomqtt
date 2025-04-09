package com.correomqtt.plugin.ui.connection

import com.intellij.icons.AllIcons
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.DataContext
import com.intellij.openapi.project.DumbAwareAction
import com.intellij.openapi.project.Project
import com.intellij.openapi.ui.ComboBox
import com.intellij.openapi.ui.DialogWrapper
import com.intellij.ui.components.JBCheckBox
import com.intellij.ui.components.JBLabel
import com.intellij.ui.components.JBTextField
import com.intellij.util.ui.GridBag
import org.correomqtt.core.model.ConnectionConfigDTO
import org.correomqtt.core.model.CorreoMqttVersion
import java.awt.BorderLayout
import java.awt.GridBagConstraints
import java.awt.GridBagLayout
import java.util.*
import javax.swing.JButton
import javax.swing.JComponent
import javax.swing.JPanel
import javax.swing.border.EmptyBorder

class AddConnectionDialog(project: Project) : DialogWrapper(project) {
    private val nameField = JBTextField()
    private val urlField = JBTextField()
    private val portField = JBTextField()
    private val clientIdField = JBTextField()
    private val userNameField = JBTextField()
    private val passwordField = JBTextField()
    private val cleanSessionField = JBCheckBox()
    private val mqttVersionField = ComboBox<String>().apply {
        addItem(CorreoMqttVersion.MQTT_3_1_1.toString())
        addItem(CorreoMqttVersion.MQTT_5_0.toString())
    }

    init {
        init() // Initialize the dialog
        title = "Add Correo Connection"
        isOKActionEnabled = true
        setSize(600, 400)
    }

    override fun createCenterPanel(): JComponent {
        val root = JPanel(BorderLayout())
        val panel = JPanel(GridBagLayout())
        val gb = GridBag()

        val checkBoxContainer = JPanel(BorderLayout())
        cleanSessionField.border = EmptyBorder(0, 0, 0, 8)
        checkBoxContainer.add(cleanSessionField, BorderLayout.WEST)
        checkBoxContainer.add(JBLabel("Clean Session"))

        // Add rows with labels and text fields
        addRow(panel, gb, "Name", nameField)
        addRow(panel, gb, "URL", urlField)
        addRow(panel, gb, "Port", portField)

        val clientIdContainer = JPanel(BorderLayout())
        val generateButton = JButton("Generate")

        val generateUUIDAction = object : DumbAwareAction(
            "",
            "",
            AllIcons.Actions.Execute
        ) {
            override fun actionPerformed(e: AnActionEvent) {
                val myUuid = UUID.randomUUID()
                clientIdField.text = myUuid.toString()
                clientIdField.revalidate()
                clientIdField.repaint()
            }
        }
        generateButton.addActionListener {
            generateUUIDAction.actionPerformed(
                AnActionEvent.createFromAnAction(
                    generateUUIDAction,
                    null,
                    "",
                    DataContext.EMPTY_CONTEXT
                )
            )
        }

        clientIdContainer.add(generateButton, BorderLayout.WEST)
        clientIdContainer.add(clientIdField, BorderLayout.CENTER)

        panel.add(
            JBLabel("Client ID"),
            gb.nextLine().next().weightx(0.2).anchor(GridBagConstraints.WEST).insetBottom(16)
        )
        panel.add(clientIdContainer, gb.next().weightx(0.6).fillCellHorizontally().insetBottom(16))

        addRow(panel, gb, "Username", userNameField)
        addRow(panel, gb, "Password", passwordField)
        addRow(panel, gb, "MQTT-Version", mqttVersionField)

        panel.add(JPanel(), gb.nextLine().next().weightx(0.2).anchor(GridBagConstraints.WEST).insetBottom(16))
        panel.add(checkBoxContainer, gb.next().weightx(0.6).fillCellHorizontally().insetBottom(16))

        root.add(panel, BorderLayout.NORTH)

        return root
    }

    private fun addRow(panel: JPanel, gb: GridBag, labelText: String, component: JComponent, switch: Boolean = false) {
        panel.add(JBLabel(labelText), gb.nextLine().next().weightx(0.2).anchor(GridBagConstraints.WEST).insetBottom(16))
        panel.add(component, gb.next().weightx(0.6).fillCellHorizontally().insetBottom(16))
    }

    fun getConnectionDTO(): ConnectionConfigDTO {
        val mqttVersion: CorreoMqttVersion = when (mqttVersionField.selectedItem) {
            CorreoMqttVersion.MQTT_3_1_1.toString() -> CorreoMqttVersion.MQTT_3_1_1
            CorreoMqttVersion.MQTT_5_0.toString() -> CorreoMqttVersion.MQTT_5_0
            else -> CorreoMqttVersion.MQTT_3_1_1
        }

        return ConnectionConfigDTO.builder()
            .id(UUID.randomUUID().toString())
            .name(nameField.text)
            .url(urlField.text)
            .port(portField.text.toInt())
            .clientId(clientIdField.text)
            .username(userNameField.text)
            .password(passwordField.text)
            .cleanSession(cleanSessionField.isSelected)
            .mqttVersion(mqttVersion)
            .build()
    }
}