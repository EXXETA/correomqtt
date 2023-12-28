package org.correomqtt.gui.controller;

import javafx.fxml.FXML;
import javafx.scene.layout.HBox;
import javafx.stage.FileChooser;
import javafx.stage.Stage;
import org.correomqtt.business.dispatcher.ImportConnectionsFileDispatcher;
import org.correomqtt.business.dispatcher.ImportConnectionsFileObserver;
import org.correomqtt.business.model.ConnectionExportDTO;
import org.correomqtt.gui.business.ExportTaskFactory;
import org.correomqtt.gui.helper.AlertHelper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.util.ResourceBundle;

public class ConnectionImportStepChooseFileViewController extends BaseControllerImpl implements ImportConnectionsFileObserver, ConnectionImportStepController {
    private static final Logger LOGGER = LoggerFactory.getLogger(ConnectionImportStepChooseFileViewController.class);
    private static ResourceBundle resources;
    private final ConnectionImportStepDelegate delegate;
    @FXML
    private HBox stepHolder;
    private File file;

    public ConnectionImportStepChooseFileViewController(ConnectionImportStepDelegate delegate) {
        this.delegate = delegate;
        ImportConnectionsFileDispatcher.getInstance().addObserver(this);
    }

    public static LoaderResult<ConnectionImportStepChooseFileViewController> load(ConnectionImportStepDelegate delegate) {
        LoaderResult<ConnectionImportStepChooseFileViewController> result = load(ConnectionImportStepChooseFileViewController.class, "connectionImportStepChooseFile.fxml",
                () -> new ConnectionImportStepChooseFileViewController(delegate));
        resources = result.getResourceBundle();
        return result;
    }

    public void choseFile() {
        Stage stage = (Stage) stepHolder.getScene().getWindow();

        FileChooser fileChooser = new FileChooser();
        fileChooser.setTitle(resources.getString("importUtilsTitle"));
        FileChooser.ExtensionFilter extFilter = new FileChooser.ExtensionFilter(resources.getString("importUtilsDescription"), "*.cqc");
        fileChooser.getExtensionFilters().add(extFilter);

        file = fileChooser.showOpenDialog(stage);
        if (file != null) {
            ExportTaskFactory.importConnectionsFile(file);
        }
    }

    @Override
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
            onImportFailed(file, null);
        }
    }

    @Override
    public void onImportCancelled(File file) {
        this.onImportFailed(file,null);
    }

    @Override
    public void onImportFailed(File file, Throwable exception) {
        AlertHelper.warn(resources.getString("connectionImportFileFailedTitle"),
                resources.getString("connectionImportFileFailedDescription"));
        delegate.onCancelClicked();
    }

    public void onCancelClicked() {
        this.delegate.onCancelClicked();
    }

    @Override
    public void cleanUp() {
        ImportConnectionsFileDispatcher.getInstance().removeObserver(this);
    }

    @Override
    public void initFromWizard() {
        // nothing to do
    }
}
