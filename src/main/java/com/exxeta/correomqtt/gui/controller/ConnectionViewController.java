package com.exxeta.correomqtt.gui.controller;

import com.exxeta.correomqtt.business.dispatcher.ConnectionLifecycleDispatcher;
import com.exxeta.correomqtt.business.dispatcher.ConnectionLifecycleObserver;
import com.exxeta.correomqtt.business.dispatcher.ExportMessageDispatcher;
import com.exxeta.correomqtt.business.dispatcher.ExportMessageObserver;
import com.exxeta.correomqtt.business.dispatcher.ImportMessageDispatcher;
import com.exxeta.correomqtt.business.dispatcher.ImportMessageObserver;
import com.exxeta.correomqtt.business.dispatcher.LogDispatcher;
import com.exxeta.correomqtt.business.dispatcher.LogObserver;
import com.exxeta.correomqtt.business.model.MessageDTO;
import com.exxeta.correomqtt.gui.business.TaskFactory;
import com.exxeta.correomqtt.gui.helper.AlertHelper;
import com.exxeta.correomqtt.gui.model.ConnectionPropertiesDTO;
import com.exxeta.correomqtt.gui.model.ConnectionState;
import com.exxeta.correomqtt.gui.model.MessagePropertiesDTO;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.SplitPane;
import javafx.scene.layout.Pane;
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
    private boolean splitMessageDetails;
    private Pane publishPane;
    private Pane subscribePane;
    private Pane controlBarPane;
    private LoadingViewController loadingViewController;
    private PublishViewController publishController;
    private static ResourceBundle resources;

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
        LoaderResult<PublishViewController> publishLoadResult = PublishViewController.load(getConnectionId(), this);
        LoaderResult<SubscriptionViewController> subscriptionLoadResult = SubscriptionViewController.load(getConnectionId(), this);
        LoaderResult<ControlBarController> controlBarLoadResult = ControlBarController.load(getConnectionId(), this);

        publishPane = publishLoadResult.getMainPane();
        subscribePane = subscriptionLoadResult.getMainPane();
        controlBarPane = controlBarLoadResult.getMainPane();

        publishController = publishLoadResult.getController();
        resources = controlBarLoadResult.getResourceBundle();

        connectionHolder.getChildren().add(0, controlBarPane);
        splitPane.getItems().add(publishPane);
        splitPane.getItems().add(subscribePane);
    }

    @Override
    public void updateLog(String message) {
        // do nothing
    }

    @Override
    public void setLayout(boolean publish, boolean subscribe) {
        if (publish && !subscribe) {
            splitPane.getItems().remove(subscribePane);
            if (!splitPane.getItems().contains(publishPane)) {
                splitPane.getItems().add(0, publishPane);
            }
        } else if (!publish && subscribe) {
            splitPane.getItems().remove(publishPane);
            if (!splitPane.getItems().contains(subscribePane)) {
                splitPane.getItems().add(0, subscribePane);
            }
        } else {
            if (!splitPane.getItems().contains(subscribePane)) {
                splitPane.getItems().add(subscribePane);
            }
            if (!splitPane.getItems().contains(publishPane)) {
                splitPane.getItems().add(0, publishPane);
            }
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

    }

    public void disconnect() {
        TaskFactory.disconnect(getConnectionId());
    }

    public Pane getMainNode() {
        return connectionHolder;
    }

    public void connect(ConnectionPropertiesDTO config) {
        splitPane.setDisable(true);
        this.setConnectionId(config.getId());
        TaskFactory.connect(getConnectionId());
    }

    @Override
    public void onExportStarted(File file, MessageDTO messageDTO) {
        Platform.runLater(() -> {
            splitPane.setDisable(true);
            loadingViewController = LoadingViewController.showAsDialog(getConnectionId(), resources.getString("connectionViewControllerExportTitle") + " " + file.getAbsolutePath());
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
        Platform.runLater(() -> {
            splitPane.setDisable(true);
        });
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
