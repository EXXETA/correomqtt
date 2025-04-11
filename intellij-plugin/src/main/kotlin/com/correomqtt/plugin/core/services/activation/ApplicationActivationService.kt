package com.correomqtt.plugin.core.services.activation

interface ApplicationActivationService {
    /**
     * Initialization of correo core functionality.
     * Scans classpath which for Soy DI Library.
     */
    fun init()
}
