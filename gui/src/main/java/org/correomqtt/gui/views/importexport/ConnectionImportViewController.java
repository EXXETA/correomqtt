package org.correomqtt.gui.views.importexport;

import javafx.fxml.FXML;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.Region;
import javafx.scene.layout.VBox;
import javafx.stage.Stage;
import lombok.AllArgsConstructor;
import org.correomqtt.core.settings.SettingsProvider;
import org.correomqtt.core.model.ConnectionConfigDTO;
import org.correomqtt.core.model.ConnectionExportDTO;
import org.correomqtt.gui.model.WindowProperty;
import org.correomqtt.gui.model.WindowType;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.utils.WindowHelper;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import javax.inject.Inject;
import java.util.EnumMap;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.ResourceBundle;
import java.util.function.Supplier;

public class ConnectionImportViewController extends BaseControllerImpl implements ConnectionImportStepDelegate {
    private static final Logger LOGGER = LoggerFactory.getLogger(ConnectionImportViewController.class);
    private final Map<Step, StepState> stepStates = new EnumMap<>(Step.class);
    private final ConnectionImportStepChooseFileViewController.Factory importChooseFileCtrlFactory;
    private final ConnectionImportStepConnectionsViewController.Factory importConnectionsCtrlFactory;
    private final ConnectionImportStepDecryptViewController.Factory importDecryptCtrlFactory;
    private final ConnectionImportStepFinalViewController.Factory importFinalCtrlFactory;
    private List<ConnectionConfigDTO> originalImportedConnections;
    private List<ConnectionConfigDTO> importableConnections;
    @FXML
    private VBox contentHolder;
    private ConnectionExportDTO originalImportedDTO;

    @Inject
    protected ConnectionImportViewController(SettingsProvider settingsProvider,
                                             ThemeManager themeManager,
                                             ConnectionImportStepChooseFileViewController.Factory importChooseFileCtrlFactory,
                                             ConnectionImportStepConnectionsViewController.Factory importConnectionsCtrlFactory,
                                             ConnectionImportStepDecryptViewController.Factory importDecryptCtrlFactory,
                                             ConnectionImportStepFinalViewController.Factory importFinalCtrlFactory
    ) {
        super(settingsProvider, themeManager);
        this.importChooseFileCtrlFactory = importChooseFileCtrlFactory;
        this.importConnectionsCtrlFactory = importConnectionsCtrlFactory;
        this.importDecryptCtrlFactory = importDecryptCtrlFactory;
        this.importFinalCtrlFactory = importFinalCtrlFactory;
    }

    public void showAsDialog() {
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

    public LoaderResult<ConnectionImportViewController> load() {
        return load(ConnectionImportViewController.class, "connectionImportView.fxml",
                () -> this);
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

    private void cleanUp() {
        stepStates.forEach((k, s) -> s.controller.cleanUp());
    }

    @FXML
    private void initialize() {
        goStepChooseFile();
    }

    @Override
    public void setOriginalImportedDTO(ConnectionExportDTO originalImportedDTO) {
        this.originalImportedDTO = originalImportedDTO;
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

    @Override
    public void goStepDecrypt() {
        activateStep(Step.DECRYPT, () -> importDecryptCtrlFactory.create(this).load());
    }

    private enum Step {CHOOSE_FILE, DECRYPT, CONNECTIONS, FINAL}

    @Override
    public void goStepConnections() {
        activateStep(Step.CONNECTIONS, () -> importConnectionsCtrlFactory.create(this).load());
    }

    @AllArgsConstructor
    private static class StepState {
        private ConnectionImportStepController controller;
        private Region region;
    }

    @Override
    public void goStepChooseFile() {
        activateStep(Step.CHOOSE_FILE, () -> importChooseFileCtrlFactory.create(this).load());
    }

    @Override
    public void goStepFinal() {
        activateStep(Step.FINAL, () -> importFinalCtrlFactory.create(this).load());
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


}
