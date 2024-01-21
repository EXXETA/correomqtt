package org.correomqtt.gui.views.importexport;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedInject;
import javafx.fxml.FXML;
import javafx.scene.control.PasswordField;
import javafx.scene.control.Tooltip;
import org.correomqtt.core.settings.SettingsProvider;
import org.correomqtt.core.concurrent.TaskErrorResult;
import org.correomqtt.core.importexport.connections.ImportDecryptConnectionsTask;
import org.correomqtt.core.model.ConnectionConfigDTO;
import org.correomqtt.core.model.ConnectionExportDTO;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;

import java.util.List;
import java.util.ResourceBundle;

public class ConnectionImportStepDecryptViewController extends BaseControllerImpl implements ConnectionImportStepController {
    private static ResourceBundle resources;

    private final AlertHelper alertHelper;
    private final ConnectionImportStepDelegate delegate;
    @FXML
    private PasswordField passwordField;

    private static final String EXCLAMATION_CIRCLE_SOLID = "exclamationCircleSolid";

    @AssistedInject
    public ConnectionImportStepDecryptViewController(
            SettingsProvider settingsProvider,
            ThemeManager themeManager,
            AlertHelper alertHelper,
            @Assisted ConnectionImportStepDelegate delegate) {
        super(settingsProvider, themeManager);
        this.alertHelper = alertHelper;
        this.delegate = delegate;
    }

    public LoaderResult<ConnectionImportStepDecryptViewController> load() {
        LoaderResult<ConnectionImportStepDecryptViewController> result = load(
                ConnectionImportStepDecryptViewController.class,
                "connectionImportStepDecrypt.fxml",
                () -> this);
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
        alertHelper.warn(resources.getString("connectionImportDecryptFailedTitle"),
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
