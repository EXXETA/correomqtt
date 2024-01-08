package org.correomqtt.business.scripting;

public record ScriptExecutionError(Type type, Throwable throwable) {

    public enum Type {
        GUEST, HOST
    }
}
