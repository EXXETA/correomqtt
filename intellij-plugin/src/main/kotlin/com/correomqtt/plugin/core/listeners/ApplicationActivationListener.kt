package com.correomqtt.plugin.core.listeners

import com.correomqtt.plugin.core.services.activation.ApplicationActivationService
import com.intellij.openapi.application.ApplicationActivationListener
import com.intellij.openapi.components.service
import com.intellij.openapi.wm.IdeFrame

/**
 * Entry point of the plugin and initializes core functionality.
 */
internal class ApplicationActivationListener : ApplicationActivationListener {

    private val service = service<ApplicationActivationService>()

    override fun applicationActivated(ideFrame: IdeFrame) {
        service.init()
    }
}
