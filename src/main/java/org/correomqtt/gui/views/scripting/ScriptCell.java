package org.correomqtt.gui.views.scripting;

import javafx.animation.Animation;
import javafx.animation.Interpolator;
import javafx.animation.RotateTransition;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.layout.Pane;
import javafx.scene.transform.Rotate;
import javafx.util.Duration;
import org.correomqtt.business.fileprovider.SettingsProvider;
import org.correomqtt.business.scripting.ScriptingBackend;
import org.correomqtt.gui.controls.ThemedFontIcon;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.text.MessageFormat;
import java.util.Comparator;
import java.util.List;
import java.util.Map;
import java.util.ResourceBundle;

import static java.util.stream.Collectors.counting;
import static java.util.stream.Collectors.groupingBy;

public class ScriptCell extends ListCell<ScriptFilePropertiesDTO> {

    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptCell.class);
    private final ListView<ScriptFilePropertiesDTO> listView;

    @FXML
    private Pane mainNode;
    @FXML
    private Label nameLabel;

    @FXML
    private Label descriptionLabel;
    @FXML
    private ResourceBundle resources;

    @FXML
    private ThemedFontIcon themedIcon;
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

    public ScriptCell(ListView<ScriptFilePropertiesDTO> listView) {
        this.listView = listView;
    }

    @Override
    protected void updateItem(ScriptFilePropertiesDTO scriptingDTO, boolean empty) {
        super.updateItem(scriptingDTO, empty);

        if (empty || scriptingDTO == null) {
            setText(null);
            setGraphic(null);
        } else {

            if (loader == null) {
                try {
                    loader = new FXMLLoader(ScriptCell.class.getResource("scriptCell.fxml"),
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
            setScript(scriptingDTO);
            setText(null);
            setGraphic(mainNode);
        }
    }

    private void setScript(ScriptFilePropertiesDTO scriptingDTO) {

        nameLabel.setText(scriptingDTO.getName());

        List<ExecutionPropertiesDTO> executions = ScriptingBackend.getInstance().getExecutions().stream()
                .filter(e -> scriptingDTO.getName().equals(e.getScriptFile().getName()))
                .map(ExecutionTransformer::dtoToProps)
                .toList();

        if (executions.isEmpty()) {
            descriptionLabel.setText(resources.getString("scriptingCellNoExecutionsYet"));
            themedIcon.setIconLiteral("mdi-script");
            rotateTransition.stop();
            themedIcon.setRotate(0);
        } else {
            long running = executions.stream()
                    .filter(e -> e.getState() == ScriptState.RUNNING)
                    .count();

            String description;
            if (running == 0) {
                description = MessageFormat.format("{0} finished", executions.size());
            } else {
                description = MessageFormat.format("{0} running / {1} finished", running, executions.size() - running);
            }
            descriptionLabel.setText(description);

            ExecutionPropertiesDTO lastExecution = executions.stream()
                    .max(Comparator.comparing(ExecutionPropertiesDTO::getSortTime))
                    .orElseThrow();

            ScriptState state = lastExecution.getState();

            if (state.isAnimation()) {
                rotateTransition.play();
            } else {
                themedIcon.setRotate(0);
                rotateTransition.stop();
            }

            themedIcon.setIconLiteral(state.getIcon());

        }
    }
}
