package org.correomqtt.gui.views;

import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.layout.AnchorPane;
import org.correomqtt.business.fileprovider.SettingsProvider;

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
        preloaderAnchorPane.getStyleClass().add(SettingsProvider.getInstance().getIconModeCssClass());
    }

    public Label getPreloaderProgressLabel() { return preloaderProgressLabel; }
    public Label getPreloaderStepLabel() {
        return preloaderStepLabel;
    }
    public Label getPreloaderVersionLabel() { return preloaderVersionLabel; }
}
