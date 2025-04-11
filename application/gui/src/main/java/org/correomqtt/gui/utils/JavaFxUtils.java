package org.correomqtt.gui.utils;

import javafx.scene.Node;
import javafx.scene.control.SplitPane;

public class JavaFxUtils {

    private JavaFxUtils(){
        // private constructor
    }
    public static void addSafeToSplitPane(SplitPane container, Node node) {
        if (!container.getItems().contains(node)) {
            container.getItems().add(node);
        }
    }
}
