package com.correomqtt.plugin

import com.correomqtt.plugin.core.services.secruity.KeyringManager
import com.intellij.openapi.diagnostic.thisLogger
import org.correomqtt.core.CorreoCore
import org.correomqtt.di.Inject
import org.correomqtt.di.SingletonBean

/**
 * Core plugin object which is called during plugin initialization.
 * - correoCore -> Correo Backend initialization
 * - keyringManager -> Intellij keyring and password encryption.
 */
@SingletonBean
class CorreoPlugin @Inject constructor(
    private var correoCore: CorreoCore,
    private var keyringManager: KeyringManager
) {
    private var isInitialized = false;

    fun init() {
        if (!isInitialized) {
            thisLogger().info("Correo App init")
            correoCore!!.init()
            keyringManager.init()
            isInitialized = true;
        }
    }
}
