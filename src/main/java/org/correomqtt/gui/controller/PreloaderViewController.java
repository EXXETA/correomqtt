package org.correomqtt.gui.controller;

import javafx.fxml.FXML;
import javafx.scene.control.Label;

public class PreloaderViewController {

    @FXML
    private Label preloaderProgressLabel;
    @FXML
    private Label preloaderStepLabel;
    @FXML
    private Label versionLabel;

    public Label getPreloaderProgressLabel() { return preloaderProgressLabel; }
    public Label getPreloaderStepLabel() {
        return preloaderStepLabel;
    }
    public Label getVersionLabel() { return versionLabel; }
}
