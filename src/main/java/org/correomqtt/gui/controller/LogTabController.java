package org.correomqtt.gui.controller;

import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.layout.AnchorPane;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.eventbus.Subscribe;
import org.correomqtt.business.log.LogEvent;
import org.correomqtt.business.log.PopLogCache;
import org.fxmisc.richtext.CodeArea;

public class LogTabController extends BaseControllerImpl {

    @FXML
    public AnchorPane logViewAnchor;
    @FXML
    public CodeArea logTextArea;
    @FXML
    private Button trashButton;

    public LogTabController() {
        EventBus.register(this);
    }

    public static LoaderResult<LogTabController> load() {
        return load(LogTabController.class, "logView.fxml");
    }

    @FXML
    private void initialize() {
        trashButton.setOnAction(event -> logTextArea.clear());
        EventBus.fire(new PopLogCache());
    }

    @SuppressWarnings("unused")
    public void updateLog(@Subscribe LogEvent event) {
        String[] matches = event.logMsg().split("\u001B");
        String cssClass;
        for (String match : matches) {
            String str;
                if(match.startsWith("[36m")) {
                    cssClass = "cyan";
                    str = match.substring(4);
                } else if(match.startsWith("[34m")) {
                    cssClass = "blue";
                    str = match.substring(4);
                } else if(match.startsWith("[33m")) {
                    cssClass = "yellow";
                    str = match.substring(4);
                } else if(match.startsWith("[35m")) {
                    cssClass = "magenta";
                    str = match.substring(4);
                } else if (match.startsWith("[1;31m")) {
                    cssClass = "red";
                    str = match.substring(6);
                } else if (match.startsWith("[0;39m")) {
                    cssClass = "default";
                    str = match.substring(6);
                } else {
                    cssClass = "default";
                    str = match;
                }
            logTextArea.append(str, cssClass);
        }
        logTextArea.requestFollowCaret();
    }


    public void cleanUp() {
        EventBus.unregister(this);
    }
}
