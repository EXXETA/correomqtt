package com.correomqtt.plugin.ui.subscribe.topic

import com.intellij.util.ui.JBUI
import java.awt.*
import javax.swing.JList
import javax.swing.JPanel
import javax.swing.ListCellRenderer

class TopicListRenderer() : JPanel(), ListCellRenderer<String> {
    init {
        layout = BorderLayout()
        border = JBUI.Borders.empty(8, 16) // Inner padding for text
        isOpaque = false // Prevent default opaque painting
    }

    override fun getListCellRendererComponent(
        list: JList<out String>,
        value: String,
        index: Int,
        isSelected: Boolean,
        cellHasFocus: Boolean
    ): Component {
        return TopicItem(value, isSelected)
    }
}