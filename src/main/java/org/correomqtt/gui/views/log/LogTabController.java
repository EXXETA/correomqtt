package org.correomqtt.gui.views.log;

import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.layout.AnchorPane;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.eventbus.Subscribe;
import org.correomqtt.business.log.LogEvent;
import org.correomqtt.business.log.PopLogCache;
import org.correomqtt.gui.utils.LogAreaUtils;
import org.correomqtt.gui.views.base.BaseControllerImpl;
import org.correomqtt.gui.views.LoaderResult;
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
        LogAreaUtils.appendColorful(logTextArea,event.logMsg());
        logTextArea.requestFollowCaret();
    }


    public void cleanUp() {
        EventBus.unregister(this);
    }
}
