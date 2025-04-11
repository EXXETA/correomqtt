package org.correomqtt.core.scripting;

import java.io.Serial;
import java.io.Serializable;

public record ScriptExecutionError(Type type, String errorMsg) implements Serializable {

    @Serial
    private static final long serialVersionUID = 1905122041950251207L;

    public enum Type {
        GUEST, HOST
    }
}
