package org.correomqtt.gui.views.importexport;

import javafx.fxml.FXML;
import javafx.scene.control.PasswordField;
import javafx.scene.control.Tooltip;
import org.correomqtt.business.concurrent.TaskErrorResult;
import org.correomqtt.business.importexport.connections.ImportDecryptConnectionsTask;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.ConnectionExportDTO;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;

import java.util.List;
import java.util.ResourceBundle;

public class ConnectionImportStepDecryptViewController extends BaseControllerImpl implements ConnectionImportStepController {
    private static ResourceBundle resources;

    private final ConnectionImportStepDelegate delegate;
    @FXML
    private PasswordField passwordField;

    private static final String EXCLAMATION_CIRCLE_SOLID = "exclamationCircleSolid";

    public ConnectionImportStepDecryptViewController(ConnectionImportStepDelegate delegate) {
        this.delegate = delegate;
    }

    public static LoaderResult<ConnectionImportStepDecryptViewController> load(ConnectionImportStepDelegate delegate) {
        LoaderResult<ConnectionImportStepDecryptViewController> result = load(
                ConnectionImportStepDecryptViewController.class,
                "connectionImportStepDecrypt.fxml",
                () -> new ConnectionImportStepDecryptViewController(delegate));
        resources = result.getResourceBundle();
        return result;
    }

    public void onDecryptClicked() {
        if (passwordField.getText() == null) {
            passwordField.setTooltip(new Tooltip(resources.getString("passwordEmpty")));
            passwordField.getStyleClass().add(EXCLAMATION_CIRCLE_SOLID);
            return;
        }

        ConnectionExportDTO dto = this.delegate.getOriginalImportedDTO();
        new ImportDecryptConnectionsTask(dto.getEncryptedData(), dto.getEncryptionType(), passwordField.getText())
                .onSuccess(this::onDecryptSucceeded)
                .onError(this::onDecryptFailed);
    }

    public void onDecryptSucceeded(List<ConnectionConfigDTO> decryptedConnectionList) {
        this.delegate.setOriginalImportedConnections(decryptedConnectionList);
        this.delegate.goStepConnections();
    }

    public void onDecryptFailed(TaskErrorResult<ImportDecryptConnectionsTask.Error> result) {
        AlertHelper.warn(resources.getString("connectionImportDecryptFailedTitle"),
                resources.getString("connectionImportDecryptFailedDescription"));
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
        passwordField.clear();
    }

    public void onDecryptBackClicked() {
        this.delegate.goStepChooseFile();
    }
}
