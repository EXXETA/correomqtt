package org.correomqtt.gui.controller;

import org.correomqtt.gui.model.WindowProperty;
import org.correomqtt.gui.model.WindowType;
import org.correomqtt.gui.utils.WindowHelper;
import javafx.event.Event;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressBar;
import javafx.scene.control.ProgressIndicator;
import javafx.scene.layout.Pane;
import javafx.stage.Stage;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.HashMap;
import java.util.Map;
import java.util.ResourceBundle;


public class LoadingViewController extends BaseConnectionController {

    private static final Logger LOGGER = LoggerFactory.getLogger(LoadingViewController.class);

    private final String message;

    @FXML
    private Label progressLabel;
    @FXML
    private ProgressBar progressBar;

    @FXML
    private Pane mainPane;

    private static ResourceBundle resources;

    private LoadingViewController(String connectionId, String message) {
        super(connectionId);
        this.message = message;
    }

    public static LoadingViewController showAsDialog(final String connectionId, final String message) {

        Map<Object, Object> properties = new HashMap<>();
        properties.put(WindowProperty.WINDOW_TYPE, WindowType.LOADING);
        properties.put(WindowProperty.CONNECTION_ID, connectionId);

        if (WindowHelper.focusWindowIfAlreadyThere(properties)) {
            throw new IllegalStateException("Loading view must be used once per connection only.");
        }

        LoaderResult<LoadingViewController> result = load(LoadingViewController.class, "loadingView.fxml",
                () -> new LoadingViewController(connectionId, message));
        resources = result.getResourceBundle();

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

