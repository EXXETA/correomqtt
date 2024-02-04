package org.correomqtt.gui.views.log;

import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.layout.AnchorPane;
import lombok.Getter;
import org.correomqtt.core.CoreManager;
import org.correomqtt.di.SoyEvents;
import org.correomqtt.di.Observes;
import org.correomqtt.core.log.LogDispatchAppender;
import org.correomqtt.core.log.LogEvent;
import org.correomqtt.core.utils.LoggerUtils;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.utils.LogAreaUtils;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;
import org.fxmisc.richtext.CodeArea;

import org.correomqtt.di.Inject;

@DefaultBean
public class LogTabController extends BaseControllerImpl {

    private static final String LOG_APPENDER_GUI_NAME = "GUI";
    private final SoyEvents soyEvents;

    @Getter
    @FXML
    private AnchorPane logViewAnchor;
    @FXML
    private CodeArea logTextArea;
    @FXML
    private Button trashButton;

    @Inject
    public LogTabController(CoreManager coreManager,
                            ThemeManager themeManager,
                            SoyEvents soyEvents) {
        super(coreManager, themeManager);
        this.soyEvents = soyEvents;
    }

    public LoaderResult<LogTabController> load() {
        return load(LogTabController.class, "logView.fxml", () -> this);
    }

    @FXML
    private void initialize() {
        trashButton.setOnAction(event -> logTextArea.clear());
        LogDispatchAppender appender = (LogDispatchAppender) LoggerUtils.findLogAppender(LOG_APPENDER_GUI_NAME);
        if (appender == null) {
            throw new IllegalStateException("There is no LogAppender with name = " + LOG_APPENDER_GUI_NAME);
        }
        appender.popCache(soyEvents).forEach(msg -> LogAreaUtils.appendColorful(logTextArea, msg));
        logTextArea.requestFollowCaret();
    }

    @SuppressWarnings("unused")
    public void updateLog(@Observes LogEvent event) {
        LogAreaUtils.appendColorful(logTextArea, event.logMsg());
        logTextArea.requestFollowCaret();
    }
}
