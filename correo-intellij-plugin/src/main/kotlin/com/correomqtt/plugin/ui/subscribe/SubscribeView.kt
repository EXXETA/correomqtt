package com.correomqtt.plugin.ui.subscribe

import com.correomqtt.plugin.ui.subscribe.message.MessageListViewFactory
import com.correomqtt.plugin.ui.subscribe.payload.PayloadView
import com.correomqtt.plugin.ui.subscribe.topic.TopicListView
import com.intellij.openapi.project.Project
import com.intellij.ui.JBSplitter
import com.intellij.util.ui.JBUI
import org.correomqtt.di.Assisted
import org.correomqtt.di.DefaultBean
import org.correomqtt.di.Inject
import java.awt.BorderLayout
import javax.swing.JPanel

@DefaultBean
class SubscribeView @Inject constructor(@Assisted project: Project, topicListView: TopicListView) : JPanel(BorderLayout()) {
    private val splitter: JBSplitter = JBSplitter(false, 0.4f)
    private val mainSplitter: JBSplitter = JBSplitter(false, 0.5f)

    init {
        splitter.border = JBUI.Borders.emptyLeft(16);
        mainSplitter.border = null;

        val messageListView = MessageListViewFactory().create(project)
        val toolbar = Toolbar()
        val payloadArea = PayloadView(project)

        mainSplitter.firstComponent = topicListView
        mainSplitter.secondComponent = messageListView

        splitter.firstComponent = mainSplitter
        splitter.secondComponent = payloadArea

        val wrapperPanel = JPanel(BorderLayout()).apply {
            border = JBUI.Borders.empty(16, 0)
            add(toolbar, BorderLayout.WEST)
        }

        add(wrapperPanel, BorderLayout.NORTH)
        add(splitter, BorderLayout.CENTER)
    }
}
