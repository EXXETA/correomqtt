package org.correomqtt.gui.controller;

import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.SplitPane;
import javafx.scene.layout.Pane;
import javafx.scene.layout.Region;
import org.correomqtt.business.dispatcher.ConnectionLifecycleDispatcher;
import org.correomqtt.business.dispatcher.ConnectionLifecycleObserver;
import org.correomqtt.business.dispatcher.ExportMessageDispatcher;
import org.correomqtt.business.dispatcher.ExportMessageObserver;
import org.correomqtt.business.dispatcher.ImportMessageDispatcher;
import org.correomqtt.business.dispatcher.ImportMessageObserver;
import org.correomqtt.business.dispatcher.LogDispatcher;
import org.correomqtt.business.dispatcher.LogObserver;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.ConnectionUISettings;
import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.gui.business.MessageTaskFactory;
import org.correomqtt.gui.helper.AlertHelper;
import org.correomqtt.gui.keyring.KeyringHandler;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.model.ConnectionState;
import org.correomqtt.gui.model.MessagePropertiesDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicInteger;

public class ConnectionViewController extends BaseConnectionController implements
        LogObserver,
        ConnectionLifecycleObserver,
        ExportMessageObserver,
        ImportMessageObserver,
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

    private boolean isFinalClose;

    public ConnectionViewController(String connectionId, ConnectionViewDelegate delegate) {
        super(connectionId);
        this.delegate = delegate;
        LogDispatcher.getInstance().addObserver(this);
        ConnectionLifecycleDispatcher.getInstance().addObserver(this);
        ExportMessageDispatcher.getInstance().addObserver(this);
        ImportMessageDispatcher.getInstance().addObserver(this);
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
    public void updateLog(String message) {
        // do nothing
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

    @Override
    public void onDisconnectFromConnectionDeleted(String connectionId) {
        // do nothing
    }

    @Override
    public void onConnect() {
        Platform.runLater(() -> splitPane.setDisable(false));
    }

    @Override
    public void onConnectRunning() {
        Platform.runLater(() -> splitPane.setDisable(true));
    }

    @Override
    public void onConnectionFailed(Throwable message) {
        Platform.runLater(() -> splitPane.setDisable(true));
    }

    @Override
    public void onConnectionCanceled() {
        Platform.runLater(() -> splitPane.setDisable(true));
    }

    @Override
    public void onConnectionLost() {
        Platform.runLater(() -> splitPane.setDisable(true));
    }

    @Override
    public void onDisconnect() {
        Platform.runLater(() -> splitPane.setDisable(true));
        delegate.onDisconnect();

        if (isFinalClose) {
            delegate.onCleanup();
        }
    }

    @Override
    public void onConnectScheduled() {
        // do nothing
    }

    @Override
    public void onDisconnectCanceled() {
        // do nothing
    }

    @Override
    public void onDisconnectFailed(Throwable exception) {
        // do nothing
    }

    @Override
    public void onDisconnectRunning() {
        // do nothing
    }

    @Override
    public void onDisconnectScheduled() {
        // do nothing
    }

    @Override
    public void onConnectionReconnected() {
        splitPane.setDisable(false);
    }

    @Override
    public void onReconnectFailed(AtomicInteger triedReconnects, int maxReconnects) {
        // do nothing
    }

    public void disconnect(boolean isFinalClose) {
        this.isFinalClose = isFinalClose;
        saveConnectionUISettings();
        MessageTaskFactory.disconnect(getConnectionId());
    }

    public void cleanUp() {
        publishController.cleanUp();
        subscribeController.cleanUp();
        controlBarController.cleanUp();

        LogDispatcher.getInstance().removeObserver(this);
        ConnectionLifecycleDispatcher.getInstance().removeObserver(this);
        ExportMessageDispatcher.getInstance().removeObserver(this);
        ImportMessageDispatcher.getInstance().removeObserver(this);
    }

    public Pane getMainNode() {
        return connectionHolder;
    }

    public void connect(ConnectionPropertiesDTO config) {
        splitPane.setDisable(true);
        this.setConnectionId(config.getId());
        MessageTaskFactory.connect(getConnectionId());
    }

    @Override
    public void onExportStarted(File file, MessageDTO messageDTO) {
        Platform.runLater(() -> {
            splitPane.setDisable(true);
            loadingViewController =
                    LoadingViewController.showAsDialog(getConnectionId(),
                                                       resources.getString("connectionViewControllerExportTitle") +
                                                               " " +
                                                               file.getAbsolutePath());
        });
    }

    @Override
    public void onExportSucceeded() {
        disableLoading();
    }

    @Override
    public void onExportCancelled(File file, MessageDTO messageDTO) {
        disableLoading();
        AlertHelper.warn(resources.getString("connectionViewControllerExportCancelledTitle"),
                         resources.getString("connectionViewControllerExportCancelledContent"));
    }

    @Override
    public void onExportFailed(File file, MessageDTO messageDTO, Throwable exception) {
        disableLoading();
        AlertHelper.warn(resources.getString("connectionViewControllerExportFailedTitle"),
                         resources.getString("connectionViewControllerExportFailedContent") + exception.getLocalizedMessage());
    }

    @Override
    public void onExportRunning() {
        // do nothing
    }

    @Override
    public void onExportScheduled() {
        // do nothing
    }

    @Override
    public void onImportStarted(File file) {
        Platform.runLater(() -> splitPane.setDisable(true));
    }

    @Override
    public void onImportSucceeded(MessageDTO messageDTO) {
        disableLoading();
    }

    @Override
    public void onImportCancelled(File file) {
        disableLoading();
    }

    @Override
    public void onImportFailed(File file, Throwable exception) {
        disableLoading();
    }

    @Override
    public void onImportRunning() {
        // do nothing
    }

    @Override
    public void onImportScheduled() {
        // do nothing
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
    public void setConnectionState(ConnectionState state) {
        delegate.setConnectionState(getTabId(), state);
    }

    @Override
    public void setTabDirty() {
        delegate.setTabDirty(getTabId());
    }
}
