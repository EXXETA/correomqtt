package org.correomqtt.gui.views.importexport;

import javafx.fxml.FXML;
import javafx.scene.layout.HBox;
import javafx.stage.FileChooser;
import javafx.stage.Stage;
import org.correomqtt.core.CoreManager;
import org.correomqtt.core.concurrent.TaskErrorResult;
import org.correomqtt.core.importexport.connections.ImportConnectionsFileTask;
import org.correomqtt.core.importexport.connections.ImportConnectionsFileTaskFactory;
import org.correomqtt.core.model.ConnectionExportDTO;
import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;

import java.io.File;
import java.util.ResourceBundle;

@DefaultBean
public class ConnectionImportStepChooseFileViewController extends BaseControllerImpl implements ConnectionImportStepController {
    private ResourceBundle resources;
    private final AlertHelper alertHelper;
    private final ImportConnectionsFileTaskFactory importConnectionsFileTask;
    private final ConnectionImportStepDelegate delegate;
    @FXML
    private HBox stepHolder;



    @Inject
    public ConnectionImportStepChooseFileViewController(CoreManager coreManager,
                                                        ThemeManager themeManager,
                                                        AlertHelper alertHelper,
                                                        ImportConnectionsFileTaskFactory importConnectionsFileTask,
                                                        @Assisted ConnectionImportStepDelegate delegate) {
        super(coreManager, themeManager);
        this.alertHelper = alertHelper;
        this.importConnectionsFileTask = importConnectionsFileTask;
        this.delegate = delegate;
    }

    public LoaderResult<ConnectionImportStepChooseFileViewController> load() {
        LoaderResult<ConnectionImportStepChooseFileViewController> result = load(ConnectionImportStepChooseFileViewController.class, "connectionImportStepChooseFile.fxml",
                () -> this);
        resources = result.getResourceBundle();
        return result;
    }

    public void choseFile() {
        File file;
        Stage stage = (Stage) stepHolder.getScene().getWindow();

        FileChooser fileChooser = new FileChooser();
        fileChooser.setTitle(resources.getString("importUtilsTitle"));
        FileChooser.ExtensionFilter extFilter = new FileChooser.ExtensionFilter(resources.getString("importUtilsDescription"), "*.cqc");
        fileChooser.getExtensionFilters().add(extFilter);
        file = fileChooser.showOpenDialog(stage);

        importConnectionsFileTask.create(file)
                .onSuccess(this::onImportSucceeded)
                .onError(this::onImportError)
                .run();
    }

    public void onImportSucceeded(ConnectionExportDTO connectionExportDTO) {
        if (connectionExportDTO != null) {
            this.delegate.setOriginalImportedDTO(connectionExportDTO);
            if (connectionExportDTO.getEncryptionType() != null) {
                this.delegate.goStepDecrypt();
            } else {
                this.delegate.setOriginalImportedConnections(connectionExportDTO.getConnectionConfigDTOS());
                this.delegate.goStepConnections();
            }
        } else {
            onImportFailed();
        }
    }

    private void onImportError(TaskErrorResult<ImportConnectionsFileTask.Error> errorResult) {
        if (errorResult.isExpected()) {
            ImportConnectionsFileTask.Error expectedError = errorResult.getExpectedError();
            if (expectedError == ImportConnectionsFileTask.Error.FILE_CAN_NOT_BE_READ_OR_PARSED) {
                alertHelper.warn(resources.getString("connectionImportFileFailedTitle"),
                        resources.getString("connectionImportFileFailedDescription"));
                delegate.onCancelClicked();
            }
        } else {
            alertHelper.unexpectedAlert(errorResult.getUnexpectedError());
        }
    }

    public void onImportFailed() {
        alertHelper.warn(resources.getString("connectionImportFileFailedTitle"),
                resources.getString("connectionImportFileFailedDescription"));
        delegate.onCancelClicked();
    }

    public void onCancelClicked() {
        this.delegate.onCancelClicked();
    }

    @Override
    public void cleanUp() {
        // nothing to do
    }

    @Override
    public void initFromWizard() {
        // nothing to do
    }
}
