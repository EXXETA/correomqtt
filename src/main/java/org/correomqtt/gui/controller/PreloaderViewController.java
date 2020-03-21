package org.correomqtt.gui.controller;

import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.layout.AnchorPane;
import org.correomqtt.business.services.ConfigService;

public class PreloaderViewController {

    @FXML
    private AnchorPane preloaderAnchorPane;
    @FXML
    private Label preloaderProgressLabel;
    @FXML
    private Label preloaderStepLabel;
    @FXML
    private Label preloaderVersionLabel;

    @FXML
    private void initialize() {
        preloaderAnchorPane.getStyleClass().add(ConfigService.getInstance().getThemeSettings().getActiveTheme().getIconMode());
    }

    public Label getPreloaderProgressLabel() { return preloaderProgressLabel; }
    public Label getPreloaderStepLabel() {
        return preloaderStepLabel;
    }
    public Label getPreloaderVersionLabel() { return preloaderVersionLabel; }
}
