package com.correomqtt.plugin.ui.common.components

import com.intellij.ui.JBColor
import com.intellij.ui.components.JBLabel
import com.intellij.util.ui.JBUI
import java.awt.GridBagConstraints
import java.awt.GridBagLayout
import javax.swing.JPanel

class DefaultPanel(msg: String): JPanel(GridBagLayout()) {
    init {
        val constraints = GridBagConstraints()
        border = null
        constraints.anchor = GridBagConstraints.CENTER
        constraints.gridx = 0
        constraints.gridy = 0
        constraints.weightx = 1.0
        constraints.weighty = 1.0
        constraints.insets = JBUI.insets(10)  // Adding some padding around the text

        val defaultLabel = JBLabel(msg)
        defaultLabel.foreground = JBColor.GRAY

        add(defaultLabel, constraints)
    }
}