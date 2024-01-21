package org.correomqtt.gui.views.log;

import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.layout.AnchorPane;
import lombok.Getter;
import org.correomqtt.core.settings.SettingsProvider;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.eventbus.Subscribe;
import org.correomqtt.core.log.LogEvent;
import org.correomqtt.core.log.PopLogCache;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.utils.LogAreaUtils;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;
import org.fxmisc.richtext.CodeArea;

import javax.inject.Inject;

public class LogTabController extends BaseControllerImpl {

    @Getter
    @FXML
    private AnchorPane logViewAnchor;
    @FXML
    private CodeArea logTextArea;
    @FXML
    private Button trashButton;

    @Inject
    public LogTabController(SettingsProvider settingsProvider,
                            ThemeManager themeManager) {
        super(settingsProvider, themeManager);
        EventBus.register(this);
    }

    public LoaderResult<LogTabController> load() {
        return load(LogTabController.class, "logView.fxml", () -> this);
    }

    @FXML
    private void initialize() {
        trashButton.setOnAction(event -> logTextArea.clear());
        EventBus.fire(new PopLogCache());
    }

    @SuppressWarnings("unused")
    public void updateLog(@Subscribe LogEvent event) {
        LogAreaUtils.appendColorful(logTextArea, event.logMsg());
        logTextArea.requestFollowCaret();
    }


    public void cleanUp() {
        EventBus.unregister(this);
    }
}
