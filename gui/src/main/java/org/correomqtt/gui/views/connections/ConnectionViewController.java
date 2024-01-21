package org.correomqtt.gui.views.connections;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import dagger.assisted.AssistedInject;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.SplitPane;
import javafx.scene.layout.Pane;
import javafx.scene.layout.Region;
import org.correomqtt.core.CoreManager;
import org.correomqtt.core.connection.ConnectionLifecycleTaskFactories;
import org.correomqtt.core.connection.ConnectionStateChangedEvent;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.eventbus.Subscribe;
import org.correomqtt.core.importexport.messages.ExportMessageFailedEvent;
import org.correomqtt.core.importexport.messages.ExportMessageStartedEvent;
import org.correomqtt.core.importexport.messages.ExportMessageSuccessEvent;
import org.correomqtt.core.importexport.messages.ImportMessageFailedEvent;
import org.correomqtt.core.importexport.messages.ImportMessageStartedEvent;
import org.correomqtt.core.importexport.messages.ImportMessageSuccessEvent;
import org.correomqtt.core.model.ConnectionConfigDTO;
import org.correomqtt.core.model.ConnectionUISettings;
import org.correomqtt.gui.keyring.KeyringManager;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.model.GuiConnectionState;
import org.correomqtt.gui.model.MessagePropertiesDTO;
import org.correomqtt.gui.theme.ThemeManager;
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

    private final ConnectionLifecycleTaskFactories connectionLifecycleTaskFactories;
    private final PublishViewController.Factory publishViewControllerFactory;
    private final SubscriptionViewController.Factory subscriptionViewControllerFactory;
    private final ControlBarController.Factory controlBarControllerFactory;
    private final KeyringManager keyringManager;
    private final LoadingViewController.Factory loadingViewControllerFactory;
    private final AlertHelper alertHelper;
    private final ConnectionViewDelegate delegate;
    @FXML
    private Pane connectionPane;

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

    @AssistedFactory
    public interface Factory {
        ConnectionViewController create(String connectionId,
                                        ConnectionViewDelegate delegate);

    }

    @AssistedInject
    public ConnectionViewController(ConnectionLifecycleTaskFactories connectionLifecycleTaskFactories,
                                    PublishViewController.Factory publishViewControllerFactory,
                                    SubscriptionViewController.Factory subscriptionViewControllerFactory,
                                    ControlBarController.Factory controlBarControllerFactory,
                                    KeyringManager keyringManager,
                                    ThemeManager themeManager,
                                    LoadingViewController.Factory loadingViewControllerFactory,
                                    AlertHelper alertHelper,
                                    CoreManager coreManager,
                                    @Assisted String connectionId,
                                    @Assisted ConnectionViewDelegate delegate) {
        super(coreManager, themeManager, connectionId);
        this.connectionLifecycleTaskFactories = connectionLifecycleTaskFactories;
        this.publishViewControllerFactory = publishViewControllerFactory;
        this.subscriptionViewControllerFactory = subscriptionViewControllerFactory;
        this.controlBarControllerFactory = controlBarControllerFactory;
        this.keyringManager = keyringManager;
        this.loadingViewControllerFactory = loadingViewControllerFactory;
        this.alertHelper = alertHelper;
        this.delegate = delegate;
        EventBus.register(this);
        coreManager.getHistoryManager().activatePublishHistory(connectionId);
        coreManager.getHistoryManager().activatePublishMessageHistory(connectionId);
        coreManager.getHistoryManager().activateSubscriptionHistory(connectionId);
    }

    public LoaderResult<ConnectionViewController> load() {
        return load(ConnectionViewController.class, "connectionView.fxml", () -> this);
    }

    @FXML
    private void initialize() {
        coreManager.getSettingsManager().getConnectionConfigs().stream()
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

        LoaderResult<PublishViewController> publishLoadResult = publishViewControllerFactory.create(getConnectionId(), this).load();
        LoaderResult<SubscriptionViewController> subscriptionLoadResult = subscriptionViewControllerFactory.create(getConnectionId(), this).load();
        LoaderResult<ControlBarController> controlBarLoadResult = controlBarControllerFactory.create(getConnectionId(), this).load();

        publishRegion = publishLoadResult.getMainRegion();
        subscribeRegion = subscriptionLoadResult.getMainRegion();
        Region controlBarRegion = controlBarLoadResult.getMainRegion();

        publishController = publishLoadResult.getController();
        subscribeController = subscriptionLoadResult.getController();
        controlBarController = controlBarLoadResult.getController();
        resources = controlBarLoadResult.getResourceBundle();

        connectionPane.getChildren().add(0, controlBarRegion);

        setLayout(
                connectionConfigDTO.getConnectionUISettings().isShowPublish(),
                connectionConfigDTO.getConnectionUISettings().isShowSubscribe());

        saveConnectionUISettings();
    }

    private void setLayoutPublishOnly() {
        splitPane.getItems().remove(subscribeRegion);
        if (!splitPane.getItems().contains(publishRegion)) {
            splitPane.getItems().add(0, publishRegion);
        }
    }

    private void setLayoutSubscribeOnly() {
        splitPane.getItems().remove(publishRegion);
        if (!splitPane.getItems().contains(subscribeRegion)) {
            splitPane.getItems().add(0, subscribeRegion);
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

    public Pane getMainNode() {
        return connectionPane;
    }

    public void connect(ConnectionPropertiesDTO config) {
        splitPane.setDisable(true);
        this.setConnectionId(config.getId());
        connectionLifecycleTaskFactories.getConnectFactory().create(getConnectionId()).run();
    }

    @SuppressWarnings("unused")
    public void onExportStarted(@Subscribe ExportMessageStartedEvent event) {
        Platform.runLater(() -> {
            splitPane.setDisable(true);
            loadingViewController = loadingViewControllerFactory.create(getConnectionId(),
                    resources.getString("connectionViewControllerExportTitle") +
                            " " +
                            event.file().getAbsolutePath()).showAsDialog();
        });
    }

    @SuppressWarnings("unused")
    @Subscribe(ExportMessageSuccessEvent.class)
    public void onExportSucceeded() {
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

    @SuppressWarnings("unused")
    public void onExportFailed(@Subscribe ExportMessageFailedEvent event) {
        disableLoading();
        alertHelper.warn(resources.getString("connectionViewControllerExportFailedTitle"),
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

        keyringManager.retryWithMasterPassword(
                masterPassword -> coreManager.getSettingsManager()
                        .saveConnections(coreManager.getSettingsManager().getConnectionConfigs(), masterPassword),
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

    @Override
    public void setTabDirty() {
        delegate.setTabDirty(getTabId());
    }

    public void close() {
        this.disconnect();
    }

    public void disconnect() {
        saveConnectionUISettings();
        connectionLifecycleTaskFactories.getDisconnectFactory().create(getConnectionId())
                .onFinally(this::cleanUp)
                .run();
    }

    public void cleanUp() {
        publishController.cleanUp();
        subscribeController.cleanUp();
        controlBarController.cleanUp();

        coreManager.getHistoryManager().tearDownPublishHistory(connectionId);
        coreManager.getHistoryManager().tearDownPublishMessageHistory(connectionId);
        coreManager.getHistoryManager().tearDownSubscriptionHistory(connectionId);

        EventBus.unregister(this);
    }
}
