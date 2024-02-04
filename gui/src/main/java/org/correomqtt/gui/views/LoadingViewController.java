package org.correomqtt.gui.views;

import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import javafx.event.Event;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressBar;
import javafx.scene.control.ProgressIndicator;
import javafx.scene.layout.Pane;
import javafx.stage.Stage;
import org.correomqtt.core.CoreManager;
import org.correomqtt.gui.model.WindowProperty;
import org.correomqtt.gui.model.WindowType;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.utils.WindowHelper;
import org.correomqtt.gui.views.connections.BaseConnectionController;

import java.util.HashMap;
import java.util.Map;
import java.util.ResourceBundle;

@DefaultBean
public class LoadingViewController extends BaseConnectionController {

    private final String message;

    @FXML
    private Label progressLabel;
    @FXML
    private ProgressBar progressBar;

    @FXML
    private Pane mainPane;


    @Inject
    LoadingViewController(CoreManager coreManager,
                          ThemeManager themeManager,
                          @Assisted("connectionId") String connectionId,
                          @Assisted("message") String message) {
        super(coreManager, themeManager, connectionId);
        this.message = message;
    }

    public LoadingViewController showAsDialog() {

        Map<Object, Object> properties = new HashMap<>();
        properties.put(WindowProperty.WINDOW_TYPE, WindowType.LOADING);
        properties.put(WindowProperty.CONNECTION_ID, connectionId);

        if (WindowHelper.focusWindowIfAlreadyThere(properties)) {
            throw new IllegalStateException("Loading view must be used once per connection only.");
        }

        LoaderResult<LoadingViewController> result = load(LoadingViewController.class, "loadingView.fxml",
                () -> this);
        ResourceBundle resources = result.getResourceBundle();

        showAsDialog(result,
                resources.getString("loadingViewControllerWait"),
                properties,
                false,
                true,
                Event::consume,
                null);

        return result.getController();

    }

    public void close() {
        Stage stage = (Stage) mainPane.getScene().getWindow();
        stage.close();
    }

    @FXML
    private void initialize() {
        progressLabel.setText(message);
        progressBar.setProgress(ProgressIndicator.INDETERMINATE_PROGRESS);
    }

    public void setProgress(int progress) {
        progressBar.setProgress(progress);
    }
}

