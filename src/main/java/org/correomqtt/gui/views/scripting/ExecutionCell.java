package org.correomqtt.gui.views.scripting;

import javafx.animation.Animation;
import javafx.animation.Interpolator;
import javafx.animation.RotateTransition;
import javafx.beans.value.ObservableValue;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.layout.Pane;
import javafx.scene.transform.Rotate;
import javafx.util.Duration;
import org.correomqtt.business.fileprovider.SettingsProvider;
import org.correomqtt.gui.controls.ThemedFontIcon;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.time.format.DateTimeFormatter;
import java.util.ResourceBundle;

public class ExecutionCell extends ListCell<ExecutionPropertiesDTO> {

    private static final Logger LOGGER = LoggerFactory.getLogger(ExecutionCell.class);
    private final ListView<ExecutionPropertiesDTO> listView;

    @FXML
    private Pane mainNode;
    @FXML
    private Label nameLabel;
    @FXML
    private Label descriptionLabel;

    @FXML
    private Label executionTimeLabel;
    @FXML
    private ThemedFontIcon themedIcon;
    @FXML
    private ResourceBundle resources;

    private FXMLLoader loader;
    private RotateTransition rotateTransition;

    @FXML
    public void initialize() {
        mainNode.getStyleClass().add(SettingsProvider.getInstance().getIconModeCssClass());

        rotateTransition = new RotateTransition();
        rotateTransition.setAxis(Rotate.Z_AXIS);
        rotateTransition.setByAngle(360);
        rotateTransition.setCycleCount(Animation.INDEFINITE);
        rotateTransition.setDuration(Duration.millis(1000));
        rotateTransition.setNode(themedIcon);
        rotateTransition.setInterpolator(Interpolator.LINEAR);

    }

    public ExecutionCell(ListView<ExecutionPropertiesDTO> listView) {
        this.listView = listView;
    }

    @Override
    protected void updateItem(ExecutionPropertiesDTO executionDTO, boolean empty) {
        super.updateItem(executionDTO, empty);

        if (empty || executionDTO == null) {
            setText(null);
            setGraphic(null);
        } else {

            if (loader == null) {
                try {
                    loader = new FXMLLoader(ExecutionCell.class.getResource("executionCell.fxml"),
                            ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale()));
                    loader.setController(this);
                    loader.load();

                } catch (Exception e) {
                    LOGGER.error("Exception rendering script:", e);
                    setText(resources.getString("commonRowCreationError"));
                    setGraphic(null);
                    return;
                }

            }
            mainNode.prefWidthProperty().bind(listView.widthProperty().subtract(20));
            setScript(executionDTO);
            setText(null);
            setGraphic(mainNode);
        }
    }

    private void updateState(ObservableValue<?> observable, Object oldValue, Object newValue) {

        ExecutionPropertiesDTO dto = getItem();

        if (dto == null) {
            return;
        }

        ScriptState state = dto.getState();

        if (state.isAnimation()) {
            rotateTransition.play();
        } else {
            themedIcon.setRotate(0);
            rotateTransition.stop();
        }

        themedIcon.setIconLiteral(state.getIcon());

        if (state.isFinalState()) {
            Long time = dto.getExecutionTime();
            executionTimeLabel.textProperty().set(time == null ? "" : "(" + time + "ms)");
        }

        if (state == ScriptState.NOTSTARTED) {
            descriptionLabel.textProperty().set("Not started yet.");
        } else {
            DateTimeFormatter dtf = DateTimeFormatter.ofPattern("dd.MM.yyyy HH:mm:ss.SSS"); //TODO utility
            descriptionLabel.textProperty().set(dtf.format(dto.getStartTimeProperty().get()));
        }
    }

    private void setScript(ExecutionPropertiesDTO scriptingDTO) {

        nameLabel.textProperty().bind(scriptingDTO.getScriptFilePropertiesDTO().getNameProperty());
        updateState(null, null, null);
        scriptingDTO.getCancelledProperty().addListener(this::updateState);
        scriptingDTO.getStartTimeProperty().addListener(this::updateState);
        scriptingDTO.getExecutionTimeProperty().addListener(this::updateState);
        scriptingDTO.getErrorProperty().addListener(this::updateState);
    }
}
