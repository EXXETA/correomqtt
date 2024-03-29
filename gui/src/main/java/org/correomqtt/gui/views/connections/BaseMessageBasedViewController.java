package org.correomqtt.gui.views.connections;

import javafx.fxml.FXML;
import javafx.scene.control.SplitPane;
import org.correomqtt.core.CoreManager;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.views.LoaderResult;


public abstract class BaseMessageBasedViewController extends BaseConnectionController implements MessageListViewDelegate {


    private final MessageListViewControllerFactory messageListViewControllerFactory;
    @FXML
    protected SplitPane splitPane;

    protected MessageListViewController messageListViewController;

    protected BaseMessageBasedViewController(CoreManager coreManager,
                                             ThemeManager themeManager,
                                             MessageListViewControllerFactory messageListViewControllerFactory,
                                             String connectionId) {
        super(coreManager, themeManager, connectionId);
        this.messageListViewControllerFactory = messageListViewControllerFactory;
    }

    protected void initMessageListView() {
        LoaderResult<MessageListViewController> result = messageListViewControllerFactory.
                create(getConnectionId(), this).load();
        messageListViewController = result.getController();
        splitPane.getItems().add(messageListViewController.getMainNode());
    }

    public double getDividerPosition() {
        if (!splitPane.getDividers().isEmpty()) {
            return splitPane.getDividers().get(0).getPosition();
        } else {
            return 0.5;
        }
    }

    public double getDetailDividerPosition() {
        return messageListViewController.getDetailDividerPosition();
    }

    public boolean isDetailActive() {
        return messageListViewController.isDetailActive();
    }

    @Override
    public void setConnectionId(String connectionId) {
        super.setConnectionId(connectionId);
        messageListViewController.setConnectionId(connectionId);
    }
}
