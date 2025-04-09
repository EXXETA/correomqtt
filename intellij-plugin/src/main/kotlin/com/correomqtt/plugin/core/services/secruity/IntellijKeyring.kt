package com.correomqtt.plugin.core.services.security

import com.intellij.credentialStore.CredentialAttributes
import com.intellij.credentialStore.Credentials
import com.intellij.ide.passwordSafe.PasswordSafe
import org.correomqtt.core.keyring.BaseKeyring

class IntellijKeyring : BaseKeyring() {
    companion object {
        const val KEYRING_IDENTIFIER = "CorreoMQTT_Plugin"
    }
    private val credentialAttributes = CredentialAttributes(KEYRING_IDENTIFIER)

    override fun getPassword(p0: String?): String {
        val credentials = PasswordSafe.instance.getPassword(credentialAttributes) ?: throw Exception("No password found")
        return credentials
    }

    override fun setPassword(user: String?, password: String?) {
        val credentials = Credentials(user, password)
        PasswordSafe.instance.set(credentialAttributes, credentials)
    }

    fun wipe() {
        PasswordSafe.instance.set(credentialAttributes, null)
    }

    override fun isSupported(): Boolean {
        return true
    }

    override fun getIdentifier(): String {
        return KEYRING_IDENTIFIER;
    }

    override fun getName(): String {
       return "Intellij Keyring"
    }

    override fun getDescription(): String {
        return "Keyring implementation for Intellij"
    }
}