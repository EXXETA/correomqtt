package com.correomqtt.plugin.ui.main

import com.intellij.openapi.project.Project
import com.intellij.openapi.wm.ToolWindowFactory
import com.intellij.ui.content.ContentFactory

class MainView : ToolWindowFactory {
    override fun createToolWindowContent(project: Project, toolWindow: com.intellij.openapi.wm.ToolWindow) {
        val toolwindow = com.correomqtt.plugin.ui.main.ToolWindowFactory().create(project)
        val content = ContentFactory.getInstance().createContent(toolwindow, null, false)
        toolWindow.contentManager.addContent(content)
    }

    override fun shouldBeAvailable(project: Project) = true
}
