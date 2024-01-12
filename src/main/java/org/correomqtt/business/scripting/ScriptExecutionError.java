package org.correomqtt.business.scripting;

import java.io.Serial;
import java.io.Serializable;

public record ScriptExecutionError(Type type, Throwable throwable) implements Serializable {

    @Serial
    private static final long serialVersionUID = 1905122041950251207L;

    public enum Type {
        GUEST, HOST
    }
}
