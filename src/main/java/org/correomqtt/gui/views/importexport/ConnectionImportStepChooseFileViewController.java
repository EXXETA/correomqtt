package org.correomqtt.gui.views.importexport;

import javafx.fxml.FXML;
import javafx.scene.layout.HBox;
import javafx.stage.FileChooser;
import javafx.stage.Stage;
import org.correomqtt.business.concurrent.TaskErrorResult;
import org.correomqtt.business.importexport.connections.ImportConnectionsFileTask;
import org.correomqtt.business.model.ConnectionExportDTO;
import org.correomqtt.gui.views.base.BaseControllerImpl;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.utils.AlertHelper;

import java.io.File;
import java.util.ResourceBundle;

public class ConnectionImportStepChooseFileViewController extends BaseControllerImpl implements ConnectionImportStepController {
    private static ResourceBundle resources;
    private final ConnectionImportStepDelegate delegate;
    @FXML
    private HBox stepHolder;


    public ConnectionImportStepChooseFileViewController(ConnectionImportStepDelegate delegate) {
        this.delegate = delegate;
    }

    public static LoaderResult<ConnectionImportStepChooseFileViewController> load(ConnectionImportStepDelegate delegate) {
        LoaderResult<ConnectionImportStepChooseFileViewController> result = load(ConnectionImportStepChooseFileViewController.class, "connectionImportStepChooseFile.fxml",
                () -> new ConnectionImportStepChooseFileViewController(delegate));
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

        new ImportConnectionsFileTask(file)
                .onSuccess(this::onImportSucceeded)
                .onError(this::onImportError)
                .run();
    }

    private void onImportError(TaskErrorResult<ImportConnectionsFileTask.Error> errorResult) {
        if (errorResult.isExpected()) {
            switch (errorResult.getExpectedError()) {
                case FILE_IS_NULL -> {
                    // ignore, file dialog was aborted
                }
                case FILE_CAN_NOT_BE_READ_OR_PARSED -> {
                    AlertHelper.warn(resources.getString("connectionImportFileFailedTitle"),
                            resources.getString("connectionImportFileFailedDescription"));
                    delegate.onCancelClicked();
                }
            }
        } else {
            AlertHelper.unexpectedAlert(errorResult.getUnexpectedError());
        }
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

    public void onImportFailed() {
        AlertHelper.warn(resources.getString("connectionImportFileFailedTitle"),
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
