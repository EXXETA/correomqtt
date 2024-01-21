package org.correomqtt.gui.views.importexport;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedInject;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import org.correomqtt.core.settings.SettingsProvider;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;

import java.text.MessageFormat;
import java.util.ResourceBundle;

public class ConnectionImportStepFinalViewController extends BaseControllerImpl implements ConnectionImportStepController {

    private static ResourceBundle resources;
    private final ConnectionImportStepDelegate delegate;
    @FXML
    private Label description;

    @AssistedInject
    public ConnectionImportStepFinalViewController(
            SettingsProvider settingsProvider,
            ThemeManager themeManager,
            @Assisted ConnectionImportStepDelegate delegate) {
        super(settingsProvider, themeManager);
        this.delegate = delegate;
    }

    public LoaderResult<ConnectionImportStepFinalViewController> load() {
        LoaderResult<ConnectionImportStepFinalViewController> result = load(
                ConnectionImportStepFinalViewController.class,
                "connectionImportStepFinal.fxml",
                () -> this);
        resources = result.getResourceBundle();
        return result;
    }

    public void onCancelClicked() {
        this.delegate.onCancelClicked();
    }

    @Override
    public void initFromWizard() {
        description.setText(MessageFormat.format(
                resources.getString("connectionImportFinalDescription"),
                this.delegate.getImportableConnections().size()
        ));
    }

    @Override
    public void cleanUp() {
        // nothing to cleanup
    }

    public void onOkClicked() {
        this.delegate.onCancelClicked();
    }
}
