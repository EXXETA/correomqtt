package org.correomqtt.gui.views.importexport;

import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import org.correomqtt.core.CoreManager;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;

import java.text.MessageFormat;
import java.util.ResourceBundle;

@DefaultBean
public class ConnectionImportStepFinalViewController extends BaseControllerImpl implements ConnectionImportStepController {

    private final ConnectionImportStepDelegate delegate;
    private  ResourceBundle resources;
    @FXML
    private Label description;


    @Inject
    public ConnectionImportStepFinalViewController(CoreManager coreManager,
                                                   ThemeManager themeManager,
                                                   @Assisted ConnectionImportStepDelegate delegate) {
        super(coreManager, themeManager);
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
    public void cleanUp() {
        // nothing to cleanup
    }

    @Override
    public void initFromWizard() {
        description.setText(MessageFormat.format(
                resources.getString("connectionImportFinalDescription"),
                this.delegate.getImportableConnections().size()
        ));
    }

    public void onOkClicked() {
        this.delegate.onCancelClicked();
    }
}
