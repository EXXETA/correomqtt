package org.correomqtt.gui.views.scripting;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import dagger.assisted.AssistedInject;
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
import org.correomqtt.core.settings.SettingsManager;
import org.correomqtt.core.scripting.ScriptingBackend;
import org.correomqtt.gui.controls.ThemedFontIcon;
import org.correomqtt.gui.theme.ThemeManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.text.MessageFormat;
import java.util.Comparator;
import java.util.List;
import java.util.ResourceBundle;

public class ScriptCell extends ListCell<ScriptFilePropertiesDTO> {

    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptCell.class);
    private static final String DIRTY_CLASS = "dirty";
    private final SettingsManager settingsManager;
    private final ThemeManager themeManager;
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

    @AssistedFactory
    public interface Factory {
        ScriptCell create(ListView<ScriptFilePropertiesDTO> listView);

    }
    @AssistedInject
    public ScriptCell(SettingsManager settingsManager,
                      ThemeManager themeManager,
                      @Assisted ListView<ScriptFilePropertiesDTO> listView) {
        this.settingsManager = settingsManager;
        this.themeManager = themeManager;
        this.listView = listView;
    }

    @FXML
    private void initialize() {
        mainNode.getStyleClass().add(themeManager.getIconModeCssClass());

        rotateTransition = new RotateTransition();
        rotateTransition.setAxis(Rotate.Z_AXIS);
        rotateTransition.setByAngle(360);
        rotateTransition.setCycleCount(Animation.INDEFINITE);
        rotateTransition.setDuration(Duration.millis(1000));
        rotateTransition.setNode(themedIcon);
        rotateTransition.setInterpolator(Interpolator.LINEAR);
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
                            ResourceBundle.getBundle("org.correomqtt.i18n", settingsManager.getSettings().getCurrentLocale()));
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

        nameLabel.setText(scriptingDTO.getName() + (scriptingDTO.isDirty() ? " *" : ""));

        if (scriptingDTO.isDirty()) {
            mainNode.getStyleClass().add(DIRTY_CLASS);
        } else {
            mainNode.getStyleClass().remove(DIRTY_CLASS);
        }

        List<ExecutionPropertiesDTO> executions = ScriptingBackend.getExecutions()
                .stream()
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
