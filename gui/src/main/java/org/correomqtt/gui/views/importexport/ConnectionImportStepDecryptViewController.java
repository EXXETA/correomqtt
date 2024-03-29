package org.correomqtt.gui.views.importexport;

import javafx.fxml.FXML;
import javafx.scene.control.PasswordField;
import javafx.scene.control.Tooltip;
import org.correomqtt.core.CoreManager;
import org.correomqtt.core.concurrent.TaskErrorResult;
import org.correomqtt.core.importexport.connections.ImportDecryptConnectionsTask;
import org.correomqtt.core.importexport.connections.ImportDecryptConnectionsTaskFactory;
import org.correomqtt.core.model.ConnectionConfigDTO;
import org.correomqtt.core.model.ConnectionExportDTO;
import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;

import java.util.List;
import java.util.ResourceBundle;

@DefaultBean
public class ConnectionImportStepDecryptViewController extends BaseControllerImpl implements ConnectionImportStepController {
    private static final String EXCLAMATION_CIRCLE_SOLID = "exclamationCircleSolid";
    private final AlertHelper alertHelper;
    private final ImportDecryptConnectionsTaskFactory importDecryptConnectionsTaskFactory;
    private final ConnectionImportStepDelegate delegate;
    private ResourceBundle resources;
    @FXML
    private PasswordField passwordField;



    @Inject
    public ConnectionImportStepDecryptViewController(CoreManager coreManager,
                                                     ThemeManager themeManager,
                                                     AlertHelper alertHelper,
                                                     ImportDecryptConnectionsTaskFactory importDecryptConnectionsTaskFactory,
                                                     @Assisted ConnectionImportStepDelegate delegate) {
        super(coreManager, themeManager);
        this.alertHelper = alertHelper;
        this.importDecryptConnectionsTaskFactory = importDecryptConnectionsTaskFactory;
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
        importDecryptConnectionsTaskFactory.create(dto.getEncryptedData(), dto.getEncryptionType(), passwordField.getText())
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
