package com.correomqtt.plugin.core.services.secruity

import com.correomqtt.plugin.GuiCore
import com.correomqtt.plugin.core.services.security.IntellijKeyring
import org.correomqtt.core.model.SettingsDTO
import org.correomqtt.core.settings.SettingsManager
import org.correomqtt.di.SingletonBean
import org.correomqtt.di.SoyDi

@SingletonBean
class KeyringManager {
    private val guiCore: GuiCore = SoyDi.inject(GuiCore::class.java)
    private val settingsManager: SettingsManager = guiCore.getSettingsManager()
    private var keyring: IntellijKeyring? = null

    fun migrate(newKeyringIdentifier: String) {
        throw UnsupportedOperationException("This method will never be implemented because intellij already implements encryption.")
    }

    fun getMasterPassword(): String {
        throw UnsupportedOperationException("This method will never be implemented because intellij will never ask for the master password")
    }

    fun init() {
        val settings: SettingsDTO = settingsManager.settings
        val oldKeyringIdentifier = settings.keyringIdentifier

        keyring = null;

        if (oldKeyringIdentifier != null) {
            keyring = IntellijKeyring()
        }

        if (keyring == null) {
            keyring = IntellijKeyring()
        }

        val newKeyringIdentifier = keyring!!.identifier

        if (!newKeyringIdentifier.equals(oldKeyringIdentifier)) {
            settings.keyringIdentifier = newKeyringIdentifier
            settingsManager.saveSettings()
        }

        settingsManager.initializePasswords(IntellijKeyring.KEYRING_IDENTIFIER)
    }

    fun wipe() {
        keyring?.wipe()
    }
}