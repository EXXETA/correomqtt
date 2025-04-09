package com.correomqtt.plugin.core.services.activation

import com.correomqtt.plugin.CorreoPlugin
import com.correomqtt.plugin.MyBundle
import com.intellij.openapi.diagnostic.thisLogger
import org.correomqtt.di.SoyDi

class ApplicationActivationServiceImpl : ApplicationActivationService {

    private lateinit var app: CorreoPlugin
    private var isInitialized = false

    override fun init() {
        if (!isInitialized) {
            thisLogger().info(MyBundle.message("start"))
            SoyDi.scan("org.correomqtt")
            SoyDi.scan("com.correomqtt.plugin");
            app = SoyDi.inject(CorreoPlugin::class.java)
            app.init()

            isInitialized = true
        }
    }
}
