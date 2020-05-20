package org.correomqtt.gui.theme;

public enum IconMode {
    BLACK("black"),WHITE("white");

    private final String iconMode;

    IconMode(String white) {
        this.iconMode = white;
    }

    @Override
    public String toString() {
        return iconMode;
    }
}
