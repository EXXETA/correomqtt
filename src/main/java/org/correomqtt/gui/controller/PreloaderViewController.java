package org.correomqtt.gui.controller;

import javafx.fxml.FXML;
import javafx.scene.control.Label;

public class PreloaderViewController {
    @FXML
    private Label versionLabel;
    @FXML
    private Label preloaderStepLabel;

    public Label getVersionLabel() { return versionLabel; }
    public Label getPreloaderStepLabel() {
        return preloaderStepLabel;
    }
}
