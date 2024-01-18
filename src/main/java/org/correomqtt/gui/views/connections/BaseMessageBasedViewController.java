package org.correomqtt.gui.views.connections;

import javafx.fxml.FXML;
import javafx.scene.control.SplitPane;
import org.correomqtt.gui.views.LoaderResult;


public abstract class BaseMessageBasedViewController extends BaseConnectionController implements MessageListViewDelegate {


    @FXML
    protected SplitPane splitPane;

    protected MessageListViewController messageListViewController;

    protected BaseMessageBasedViewController(String connectionId) {
        super(connectionId);
    }

    protected void initMessageListView() {
        LoaderResult<MessageListViewController> result = MessageListViewController.load(getConnectionId(),this);
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
