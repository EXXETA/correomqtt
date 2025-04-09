package com.correomqtt.plugin.ui.tab

import com.correomqtt.plugin.ui.publish.PublishView
import com.correomqtt.plugin.ui.subscribe.SubscribeViewFactory
import com.intellij.openapi.project.Project
import com.intellij.ui.components.JBTabbedPane
import com.intellij.util.ui.JBUI
import org.correomqtt.di.Assisted
import org.correomqtt.di.DefaultBean
import org.correomqtt.di.Inject
import java.awt.BorderLayout
import javax.swing.JPanel
import javax.swing.JTabbedPane

@DefaultBean
class TabManager @Inject constructor(@Assisted var project: Project): JTabbedPane() {
    init {
        // Create the "Subscribe" tab
        val subscribeContent = com.correomqtt.plugin.ui.subscribe.SubscribeViewFactory().create(project)
        addTab("Subscribe", createTabContent(subscribeContent))

        // Create the "Publish" tab
        val publishContent = PublishView(project)
        addTab("Publish", createTabContent(publishContent))
    }

    private fun createTabContent(content: JPanel): JPanel {
        val root = JPanel(BorderLayout())
        root.border = JBUI.Borders.empty(8, 16)

        // Wrap the provided content in a JBSplitter if needed (comment/uncomment depending on your needs)
        // val splitter = JBSplitter(false, 0.5f)
        // splitter.firstComponent = content // Replace with your content as needed
        // root.add(splitter, BorderLayout.CENTER)

        root.add(content, BorderLayout.CENTER)
        return root
    }
}
