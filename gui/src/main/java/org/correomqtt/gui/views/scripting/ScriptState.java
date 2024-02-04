package org.correomqtt.gui.views.scripting;


import lombok.AllArgsConstructor;
import lombok.Getter;

@AllArgsConstructor
@Getter
public enum ScriptState {
    FAILED("mdi-alert", false, true),
    RUNNING("mdi-loading", true, false),
    CANCELLED("mdi-cancel", false, true),
    SUCCEEDED("mdi-check", false, true),
    NOTSTARTED("mdi-script", false, false);

    private final String icon;
    private final boolean animation;
    private final boolean finalState;
}