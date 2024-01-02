package org.correomqtt.gui.controller;

import javafx.fxml.FXML;
import javafx.scene.control.Label;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.text.MessageFormat;
import java.util.ResourceBundle;

public class ConnectionImportStepFinalViewController extends BaseControllerImpl implements ConnectionImportStepController {

    private static ResourceBundle resources;
    private final ConnectionImportStepDelegate delegate;
    @FXML
    public Label description;

    public ConnectionImportStepFinalViewController(ConnectionImportStepDelegate delegate) {
        this.delegate = delegate;
    }

    public static LoaderResult<ConnectionImportStepFinalViewController> load(ConnectionImportStepDelegate delegate) {
        LoaderResult<ConnectionImportStepFinalViewController> result = load(
                ConnectionImportStepFinalViewController.class,
                "connectionImportStepFinal.fxml",
                () -> new ConnectionImportStepFinalViewController(delegate));
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
