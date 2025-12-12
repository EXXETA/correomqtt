package org.correomqtt.gui.views.log;

import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.AnchorPane;
import javafx.stage.Stage;
import lombok.Getter;
import org.correomqtt.core.CoreManager;
import org.correomqtt.core.log.LogDispatchAppender;
import org.correomqtt.core.log.LogEvent;
import org.correomqtt.core.utils.LoggerUtils;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.di.Observes;
import org.correomqtt.di.SoyEvents;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.utils.FxThread;
import org.correomqtt.gui.utils.LogAreaUtils;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;
import org.fxmisc.richtext.CodeArea;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@DefaultBean
public class LogTabController extends BaseControllerImpl {

    private static final Logger LOGGER = LoggerFactory.getLogger(LogTabController.class);

    private static final String LOG_APPENDER_GUI_NAME = "GUI";

    private final SoyEvents soyEvents;

    private final LogAreaUtils logAreaUtils;

    @Getter
    @FXML
    private AnchorPane logViewAnchor;

    @FXML
    private CodeArea logTextArea;

    @FXML
    private Button trashButton;

    @FXML
    private Button logFileExportButton;

    @Inject
    public LogTabController(CoreManager coreManager,
                            ThemeManager themeManager,
                            SoyEvents soyEvents,
                            LogAreaUtils logAreaUtils) {
        super(coreManager, themeManager);
        this.soyEvents = soyEvents;
        this.logAreaUtils = logAreaUtils;
    }

    public LoaderResult<LogTabController> load() {
        return load(LogTabController.class, "logView.fxml", () -> this);
    }

    @FXML
    private void initialize() {
        trashButton.setOnAction(event -> logTextArea.clear());
        logFileExportButton.setOnMouseClicked(this::saveLogToFile);
        LogDispatchAppender appender = (LogDispatchAppender) LoggerUtils.findLogAppender(LOG_APPENDER_GUI_NAME);
        if (appender == null) {
            throw new IllegalStateException("There is no LogAppender with name = " + LOG_APPENDER_GUI_NAME);
        }
        appender.popCache(soyEvents).forEach(msg -> LogAreaUtils.appendColorful(logTextArea, msg));
        logTextArea.requestFollowCaret();
    }

    @FxThread
    @SuppressWarnings("unused")
    public void updateLog(@Observes LogEvent event) {
        LogAreaUtils.appendColorful(logTextArea, event.logMsg());
        logTextArea.requestFollowCaret();
    }

    private void saveLogToFile(MouseEvent event) {

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Clicked on save log button");
        }

        Stage stage = (Stage) logTextArea.getScene().getWindow();
        logAreaUtils.saveLogs(logTextArea.getText(), stage);
    }
}
