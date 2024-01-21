package org.correomqtt.gui.views.importexport;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import dagger.assisted.AssistedInject;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.fxml.FXML;
import javafx.scene.control.ListCell;
import javafx.scene.control.SelectionMode;
import org.controlsfx.control.CheckListView;
import org.controlsfx.control.IndexedCheckModel;
import org.correomqtt.core.settings.SettingsProvider;
import org.correomqtt.core.model.ConnectionConfigDTO;
import org.correomqtt.core.utils.ConnectionHolder;
import org.correomqtt.gui.keyring.KeyringHandler;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.transformer.ConnectionTransformer;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;

import java.util.List;
import java.util.ResourceBundle;
import java.util.stream.Stream;

public class ConnectionImportStepConnectionsViewController extends BaseControllerImpl implements ConnectionImportStepController {
    private  ResourceBundle resources;
    private final ConnectionHolder connectionHolder;
    private final SettingsProvider settingsProvider;
    private final KeyringHandler keyringHandler;
    private final AlertHelper alertHelper;
    private final ExportConnectionCell.Factory exportConnectionCellFactory;
    private final ConnectionImportStepDelegate delegate;
    @FXML
    private CheckListView<ConnectionPropertiesDTO> connectionsListView;
    private final ObservableList<ConnectionPropertiesDTO> connectionConfigDTOS = FXCollections.observableArrayList();
    private List<ConnectionPropertiesDTO> disabledConnections;

    @AssistedFactory
    public interface Factory {
        ConnectionImportStepConnectionsViewController create(ConnectionImportStepDelegate delegate);
    }
    @AssistedInject
    public ConnectionImportStepConnectionsViewController(ConnectionHolder connectionHolder,
                                                         SettingsProvider settingsProvider,
                                                         ThemeManager themeManager,
                                                         KeyringHandler keyringHandler,
                                                         AlertHelper alertHelper,
                                                         ExportConnectionCell.Factory exportConnectionCellFactory,
                                                         @Assisted ConnectionImportStepDelegate delegate) {
        super(settingsProvider, themeManager);
        this.connectionHolder = connectionHolder;
        this.settingsProvider = settingsProvider;
        this.keyringHandler = keyringHandler;
        this.alertHelper = alertHelper;
        this.exportConnectionCellFactory = exportConnectionCellFactory;
        this.delegate = delegate;
    }

    public LoaderResult<ConnectionImportStepConnectionsViewController> load() {
        LoaderResult<ConnectionImportStepConnectionsViewController> result = load(ConnectionImportStepConnectionsViewController.class, "connectionImportStepConnections.fxml",
                () -> this);
        resources = result.getResourceBundle();

        if (delegate.getOriginalImportedConnections().isEmpty()) {
            alertHelper.warn(resources.getString("connectionImportNoConnectionsTitle"),
                    resources.getString("connectionImportNoConnectionsDescription"));
            delegate.onCancelClicked();
        }
        return result;
    }

    @FXML
    private void initialize() {
        connectionsListView.getSelectionModel().setSelectionMode(SelectionMode.MULTIPLE);
    }

    private ListCell<ConnectionPropertiesDTO> createCell() {
        ExportConnectionCell cell = exportConnectionCellFactory.create(connectionsListView);
        cell.selectedProperty().addListener((observable, oldValue, newValue) ->
                connectionsListView.getSelectionModel().clearSelection());
        cell.itemProperty().addListener((observable, oldValue, newValue) ->
                cell.setDisable(disabledConnections.contains(cell.getItem())));
        return cell;
    }

    public void onCancelClicked() {
        this.delegate.onCancelClicked();
    }

    @Override
    public void cleanUp() {
        // nothing to do here
    }

    @Override
    public void initFromWizard() {
        List<ConnectionConfigDTO> importedConnections;
        importedConnections = this.delegate.getOriginalImportedConnections();
        connectionConfigDTOS.clear();
        connectionConfigDTOS.addAll(ConnectionTransformer.dtoListToPropList(importedConnections));
        connectionsListView.setItems(connectionConfigDTOS);
        List<ConnectionConfigDTO> existingConnections = connectionHolder.getSortedConnections();
        disabledConnections = connectionConfigDTOS.stream()
                .filter(cp -> existingConnections.stream()
                        .anyMatch(c -> cp.getId().equals(c.getId()) || cp.getName().equals(c.getName()))
                )
                .toList();
        connectionsListView.setCellFactory(lv -> this.createCell());
        checkAll();
    }

    public void onConnectionsBackClicked() {
        if (this.delegate.getOriginalImportedDTO().getEncryptionType() != null) {
            this.delegate.goStepDecrypt();
        } else {
            this.delegate.goStepChooseFile();
        }
    }

    public void goImportFinalStep() {

        List<ConnectionConfigDTO> importableConnections = ConnectionTransformer.propsListToDtoList(connectionsListView.getCheckModel().getCheckedItems());
        this.delegate.setImportableConnections(importableConnections);

        List<ConnectionConfigDTO> connections = Stream.concat(
                connectionHolder.getSortedConnections().stream(),
                importableConnections.stream()
        ).toList();

        keyringHandler.retryWithMasterPassword(
                masterPassword -> settingsProvider.saveConnections(connections, masterPassword),
                resources.getString("onPasswordSaveFailedTitle"),
                resources.getString("onPasswordSaveFailedHeader"),
                resources.getString("onPasswordSaveFailedContent"),
                resources.getString("onPasswordSaveFailedGiveUp"),
                resources.getString("onPasswordSaveFailedTryAgain")
        );
        this.delegate.goStepFinal();
    }

    public void checkAll() {
        IndexedCheckModel<ConnectionPropertiesDTO> checkModel = connectionsListView.getCheckModel();
        connectionsListView.getItems()
                .stream()
                .filter(c -> !disabledConnections.contains(c))
                .forEach(checkModel::check);
    }

    public void checkNone() {
        connectionsListView.getCheckModel().clearChecks();
    }
}
