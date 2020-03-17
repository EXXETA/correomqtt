package org.correomqtt.gui.controller;

import com.exxeta.correomqtt.business.dispatcher.LogDispatcher;
import com.exxeta.correomqtt.business.dispatcher.LogObserver;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.TextArea;
import javafx.scene.layout.AnchorPane;

public class LogTabController extends BaseController implements LogObserver {
    @FXML
    public AnchorPane logViewAnchor;
    @FXML
    public TextArea logTextArea;
    @FXML
    private Button trashButton;

    public LogTabController() {
        LogDispatcher.getInstance().addObserver(this);
    }

    public static LoaderResult<LogTabController> load() {
        return load(LogTabController.class, "logView.fxml");
    }

    @FXML
    private void initialize() {
        trashButton.setOnAction(event -> logTextArea.setText(""));
    }

    @Override
    public void updateLog(String message) {
        logTextArea.appendText(message);
    }

}
