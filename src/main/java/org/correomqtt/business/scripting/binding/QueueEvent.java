package org.correomqtt.business.scripting.binding;

public record QueueEvent(Runnable callback) {
    public boolean isContinue() {
        return callback == null;
    }
}
