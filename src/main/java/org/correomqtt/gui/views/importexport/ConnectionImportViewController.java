package org.correomqtt.gui.views.importexport;

import javafx.fxml.FXML;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.Region;
import javafx.scene.layout.VBox;
import javafx.stage.Stage;
import lombok.AllArgsConstructor;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.ConnectionExportDTO;
import org.correomqtt.gui.model.WindowProperty;
import org.correomqtt.gui.model.WindowType;
import org.correomqtt.gui.utils.WindowHelper;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.EnumMap;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.ResourceBundle;
import java.util.function.Supplier;

public class ConnectionImportViewController extends BaseControllerImpl implements ConnectionImportStepDelegate {
    private static final Logger LOGGER = LoggerFactory.getLogger(ConnectionImportViewController.class);
    private List<ConnectionConfigDTO> originalImportedConnections;

    private List<ConnectionConfigDTO> importableConnections;
    @AllArgsConstructor
    private static class StepState {
        private ConnectionImportStepController controller;
        private Region region;
    }

    private enum Step {CHOOSE_FILE, DECRYPT, CONNECTIONS, FINAL}

    private final Map<Step, StepState> stepStates = new EnumMap<>(Step.class);

    @FXML
    public VBox contentHolder;


    private ConnectionExportDTO originalImportedDTO;

    public static LoaderResult<ConnectionImportViewController> load() {
        return load(ConnectionImportViewController.class, "connectionImportView.fxml",
                ConnectionImportViewController::new);
    }


    public static void showAsDialog() {
        ResourceBundle resources;

        LOGGER.info("Open ConnectionImportView Dialog");
        Map<Object, Object> properties = new HashMap<>();
        properties.put(WindowProperty.WINDOW_TYPE, WindowType.CONNECTION_EXPORT);

        if (WindowHelper.focusWindowIfAlreadyThere(properties)) {
            return;
        }
        LoaderResult<ConnectionImportViewController> result = load();
        resources = result.getResourceBundle();

        showAsDialog(result, resources.getString("connectionImportViewControllerTitle"), properties, false, false, null,
                event -> result.getController().keyHandling(event));
    }


    @FXML
    public void initialize() {
        goStepChooseFile();
    }

    @Override
    public void setOriginalImportedDTO(ConnectionExportDTO originalImportedDTO) {
        this.originalImportedDTO = originalImportedDTO;
    }

    @Override
    public void goStepDecrypt() {
        activateStep(Step.DECRYPT,() -> ConnectionImportStepDecryptViewController.load(this));
    }

    @Override
    public void goStepConnections() {
        activateStep(Step.CONNECTIONS,() -> ConnectionImportStepConnectionsViewController.load(this));
    }

    @Override
    public void goStepChooseFile() {
        activateStep(Step.CHOOSE_FILE,() -> ConnectionImportStepChooseFileViewController.load(this));
    }

    @Override
    public void goStepFinal() {
        activateStep(Step.FINAL,() -> ConnectionImportStepFinalViewController.load(this));
    }

    private <Z extends ConnectionImportStepController> void activateStep(Step step, Supplier<LoaderResult<Z>> resultGenerator) {
        StepState stepState = stepStates.computeIfAbsent(step, k -> {
            LoaderResult<Z> result = resultGenerator.get();
            return new StepState(result.getController(), result.getMainRegion());
        });
        stepStates.forEach((k, s) -> contentHolder.getChildren().remove(s.region));
        stepState.controller.initFromWizard();
        contentHolder.getChildren().add(1, stepState.region);
    }

    private void keyHandling(KeyEvent event) {
        if (KeyCode.ESCAPE == event.getCode()) {
            closeDialog();
        }
    }

    private void closeDialog() {
        cleanUp();
        Stage stage = (Stage) contentHolder.getScene().getWindow();
        stage.close();
    }


    public void onCancelClicked() {
        closeDialog();
    }

    @Override
    public List<ConnectionConfigDTO> getOriginalImportedConnections() {
        return originalImportedConnections;
    }

    @Override
    public ConnectionExportDTO getOriginalImportedDTO() {
        return originalImportedDTO;
    }

    @Override
    public void setOriginalImportedConnections(List<ConnectionConfigDTO> connectionList) {
        this.originalImportedConnections = connectionList;
    }

    @Override
    public void setImportableConnections(List<ConnectionConfigDTO> connectionList) {
        this.importableConnections = connectionList;
    }

    @Override
    public List<ConnectionConfigDTO> getImportableConnections() {
        return importableConnections;
    }

    private void cleanUp() {
        stepStates.forEach((k,s) -> s.controller.cleanUp());
    }
}
