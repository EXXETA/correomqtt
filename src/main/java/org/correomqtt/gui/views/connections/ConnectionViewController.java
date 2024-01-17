package org.correomqtt.gui.views.connections;

import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.SplitPane;
import javafx.scene.layout.Pane;
import javafx.scene.layout.Region;
import org.correomqtt.business.connection.ConnectTask;
import org.correomqtt.business.connection.ConnectionStateChangedEvent;
import org.correomqtt.business.connection.DisconnectTask;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.eventbus.Subscribe;
import org.correomqtt.business.fileprovider.SettingsProvider;
import org.correomqtt.business.importexport.messages.ExportMessageFailedEvent;
import org.correomqtt.business.importexport.messages.ExportMessageStartedEvent;
import org.correomqtt.business.importexport.messages.ExportMessageSuccessEvent;
import org.correomqtt.business.importexport.messages.ImportMessageFailedEvent;
import org.correomqtt.business.importexport.messages.ImportMessageStartedEvent;
import org.correomqtt.business.importexport.messages.ImportMessageSuccessEvent;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.ConnectionUISettings;
import org.correomqtt.gui.keyring.KeyringHandler;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.model.GuiConnectionState;
import org.correomqtt.gui.model.MessagePropertiesDTO;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.LoadingViewController;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;

public class ConnectionViewController extends BaseConnectionController implements
        PublishViewDelegate,
        SubscriptionViewDelegate,
        ControlBarDelegate {

    private static final Logger LOGGER = LoggerFactory.getLogger(ConnectionViewController.class);

    private final ConnectionViewDelegate delegate;

    @FXML
    private Pane connectionHolder;

    @FXML
    private SplitPane splitPane;

    private Region publishRegion;

    private Region subscribeRegion;

    private LoadingViewController loadingViewController;

    private PublishViewController publishController;

    private ResourceBundle resources;

    private ConnectionConfigDTO connectionConfigDTO = null;

    private SubscriptionViewController subscribeController;

    private ControlBarController controlBarController;

    public ConnectionViewController(String connectionId, ConnectionViewDelegate delegate) {
        super(connectionId);
        this.delegate = delegate;
        EventBus.register(this);
    }

    public static LoaderResult<ConnectionViewController> load(String connectionId, ConnectionViewDelegate delegate) {
        return load(ConnectionViewController.class, "connectionView.fxml",
                () -> new ConnectionViewController(connectionId, delegate));
    }

    @FXML
    public void initialize() {
        SettingsProvider.getInstance().getConnectionConfigs().stream()
                .filter(c -> c.getId().equals(getConnectionId()))
                .findFirst()
                .ifPresent(c -> {
                    connectionConfigDTO = c;
                    if (connectionConfigDTO.getConnectionUISettings() == null) {
                        connectionConfigDTO.setConnectionUISettings(new ConnectionUISettings(
                                true,
                                true,
                                0.5,
                                0.5,
                                0.5,
                                false,
                                0.5,
                                0.5,
                                false
                        ));
                    }
                });

        LoaderResult<PublishViewController> publishLoadResult = PublishViewController.load(getConnectionId(), this);
        LoaderResult<SubscriptionViewController> subscriptionLoadResult = SubscriptionViewController.load(getConnectionId(), this);
        LoaderResult<ControlBarController> controlBarLoadResult = ControlBarController.load(getConnectionId(), this);

        publishRegion = publishLoadResult.getMainRegion();
        subscribeRegion = subscriptionLoadResult.getMainRegion();
        Region controlBarRegion = controlBarLoadResult.getMainRegion();

        publishController = publishLoadResult.getController();
        subscribeController = subscriptionLoadResult.getController();
        controlBarController = controlBarLoadResult.getController();
        resources = controlBarLoadResult.getResourceBundle();

        connectionHolder.getChildren().add(0, controlBarRegion);

        setLayout(
                connectionConfigDTO.getConnectionUISettings().isShowPublish(),
                connectionConfigDTO.getConnectionUISettings().isShowSubscribe());

        saveConnectionUISettings();
    }

    @Override
    public void saveConnectionUISettings() {
        LOGGER.debug("Save connection ui settings: {}", getConnectionId());
        if (!splitPane.getDividers().isEmpty()) {
            connectionConfigDTO.getConnectionUISettings()
                    .setMainDividerPosition(splitPane.getDividers().get(0).positionProperty().getValue());
            connectionConfigDTO.getConnectionUISettings().setShowPublish(true);
            connectionConfigDTO.getConnectionUISettings().setShowSubscribe(true);

        } else {
            if (splitPane.getItems().contains(publishRegion)) {
                connectionConfigDTO.getConnectionUISettings().setShowPublish(true);
                connectionConfigDTO.getConnectionUISettings().setShowSubscribe(false);
            } else if (splitPane.getItems().contains(subscribeRegion)) {
                connectionConfigDTO.getConnectionUISettings().setShowPublish(false);
                connectionConfigDTO.getConnectionUISettings().setShowSubscribe(true);
            }
        }

        connectionConfigDTO.getConnectionUISettings().setPublishDividerPosition(publishController.getDividerPosition());
        connectionConfigDTO.getConnectionUISettings().setPublishDetailDividerPosition(publishController.getDetailDividerPosition());
        connectionConfigDTO.getConnectionUISettings().setPublishDetailActive(publishController.isDetailActive());

        connectionConfigDTO.getConnectionUISettings().setSubscribeDividerPosition(subscribeController.getDividerPosition());
        connectionConfigDTO.getConnectionUISettings().setSubscribeDetailDividerPosition(subscribeController.getDetailDividerPosition());
        connectionConfigDTO.getConnectionUISettings().setSubscribeDetailActive(subscribeController.isDetailActive());

        KeyringHandler.getInstance().retryWithMasterPassword(
                masterPassword -> SettingsProvider.getInstance()
                        .saveConnections(SettingsProvider.getInstance().getConnectionConfigs(), masterPassword),
                resources.getString("onPasswordSaveFailedTitle"),
                resources.getString("onPasswordSaveFailedHeader"),
                resources.getString("onPasswordSaveFailedContent"),
                resources.getString("onPasswordSaveFailedGiveUp"),
                resources.getString("onPasswordSaveFailedTryAgain")
        );
    }

    @Override
    public void resetConnectionUISettings() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Reset connection ui settings': {}", getConnectionId());
        }

        setLayout(true, true);
        if (!splitPane.getDividers().isEmpty()) {
            splitPane.getDividers().get(0).setPosition(0.5);
        }
        if (!publishController.splitPane.getDividers().isEmpty()) {
            publishController.splitPane.getDividers().get(0).setPosition(0.5);
        }
        publishController.messageListViewController.showDetailViewButton.setSelected(false);
        publishController.messageListViewController.closeDetailView();
        if (!subscribeController.splitPane.getDividers().isEmpty()) {
            subscribeController.splitPane.getDividers().get(0).setPosition(0.5);
        }
        subscribeController.messageListViewController.showDetailViewButton.setSelected(false);
        subscribeController.messageListViewController.closeDetailView();
    }

    @Override
    public void setLayout(boolean publish, boolean subscribe) {

        if (publish && !subscribe) {
            if (LOGGER.isDebugEnabled()) {
                LOGGER.debug("Show only publish clicked': {}", getConnectionId());
            }
            setLayoutPublishOnly();
        } else if (!publish && subscribe) {
            if (LOGGER.isDebugEnabled()) {
                LOGGER.debug("Show only subscribe clicked': {}", getConnectionId());
            }
            setLayoutSubscribeOnly();
        } else {
            if (LOGGER.isDebugEnabled()) {
                LOGGER.debug("Show publish and subscribe clicked': {}", getConnectionId());
            }
            setLayoutBoth();
        }
    }

    private void setLayoutBoth() {
        if (!splitPane.getItems().contains(subscribeRegion)) {
            splitPane.getItems().add(subscribeRegion);
        }
        if (!splitPane.getItems().contains(publishRegion)) {
            splitPane.getItems().add(0, publishRegion);
        }
        if (!splitPane.getDividers().isEmpty()) {
            splitPane.getDividers()
                    .get(0)
                    .positionProperty()
                    .setValue(connectionConfigDTO.getConnectionUISettings().getMainDividerPosition());
        }
    }

    private void setLayoutSubscribeOnly() {
        splitPane.getItems().remove(publishRegion);
        if (!splitPane.getItems().contains(subscribeRegion)) {
            splitPane.getItems().add(0, subscribeRegion);
        }
    }

    private void setLayoutPublishOnly() {
        splitPane.getItems().remove(subscribeRegion);
        if (!splitPane.getItems().contains(publishRegion)) {
            splitPane.getItems().add(0, publishRegion);
        }
    }

    @SuppressWarnings("unused")
    public void onConnectionStateChanged(@Subscribe ConnectionStateChangedEvent event) {
        switch (event.getState()) {
            case CONNECTED -> splitPane.setDisable(false);
            case CONNECTING, RECONNECTING, DISCONNECTING -> splitPane.setDisable(true);
            case DISCONNECTED_GRACEFUL, DISCONNECTED_UNGRACEFUL -> {
                splitPane.setDisable(true);
                delegate.onDisconnect();
            }
        }

    }

    public void disconnect() {
        saveConnectionUISettings();
        new DisconnectTask(getConnectionId())
                .onFinally(this::cleanUp)
                .run();
    }

    public void cleanUp() {
        publishController.cleanUp();
        subscribeController.cleanUp();
        controlBarController.cleanUp();

        EventBus.unregister(this);
    }

    public Pane getMainNode() {
        return connectionHolder;
    }

    public void connect(ConnectionPropertiesDTO config) {
        splitPane.setDisable(true);
        this.setConnectionId(config.getId());
        new ConnectTask(getConnectionId()).run();
    }

    @SuppressWarnings("unused")
    public void onExportStarted(@Subscribe ExportMessageStartedEvent event) {
        Platform.runLater(() -> {
            splitPane.setDisable(true);
            loadingViewController =
                    LoadingViewController.showAsDialog(getConnectionId(),
                            resources.getString("connectionViewControllerExportTitle") +
                                    " " +
                                    event.file().getAbsolutePath());
        });
    }

    @SuppressWarnings("unused")
    @Subscribe(ExportMessageSuccessEvent.class)
    public void onExportSucceeded() {
        disableLoading();
    }

    @SuppressWarnings("unused")
    public void onExportFailed(@Subscribe ExportMessageFailedEvent event) {
        disableLoading();
        AlertHelper.warn(resources.getString("connectionViewControllerExportFailedTitle"),
                resources.getString("connectionViewControllerExportFailedContent")
                        + event.throwable().getLocalizedMessage());
    }

    @SuppressWarnings("unused")
    @Subscribe(ImportMessageStartedEvent.class)
    public void onImportStarted() {
        Platform.runLater(() -> splitPane.setDisable(true));
    }

    @SuppressWarnings("unused")
    public void onImportSucceeded(@Subscribe ImportMessageSuccessEvent event) {
        disableLoading();
    }

    @SuppressWarnings("unused")
    @Subscribe(ImportMessageFailedEvent.class)
    public void onImportFailed() {
        disableLoading();
    }

    private void disableLoading() {
        Platform.runLater(() -> {
            splitPane.setDisable(false);
            if (loadingViewController != null) {
                loadingViewController.close();
                loadingViewController = null;
            }
        });
    }

    @Override
    public void setUpToForm(MessagePropertiesDTO messageDTO) {
        if (publishController != null) {
            publishController.setUpToForm(messageDTO);
        }
    }

    @Override
    public void setConnectionState(GuiConnectionState state) {
        delegate.setConnectionState(getTabId(), state);
    }

    @Override
    public void setTabDirty() {
        delegate.setTabDirty(getTabId());
    }

    public void close() {
        this.disconnect();
    }
}
