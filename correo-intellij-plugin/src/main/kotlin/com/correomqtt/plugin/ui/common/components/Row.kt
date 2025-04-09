package com.correomqtt.plugin.ui.common.components

import com.intellij.util.ui.JBUI
import com.intellij.util.ui.UIUtil
import java.awt.BorderLayout
import java.awt.Dimension
import javax.swing.JPanel

open class Row(isSelected: Boolean): JPanel(BorderLayout()) {
    init {
        if (isSelected) {
            background = UIUtil.getListSelectionBackground(true) // Theme-aware selection color
            foreground = UIUtil.getListSelectionForeground(true) // Theme-aware selection text color
        } else {
            background = UIUtil.getListBackground() // Default background color
            foreground = UIUtil.getListForeground() // Default text color
        }

        minimumSize = Dimension(0, 48)
        maximumSize = Dimension(Int.MAX_VALUE, 48)
        border = JBUI.Borders.empty(8, 16, 8, 4)
    }
}