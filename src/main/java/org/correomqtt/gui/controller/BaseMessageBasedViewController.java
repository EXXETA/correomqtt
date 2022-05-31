package org.correomqtt.gui.controller;

import javafx.fxml.FXML;
import javafx.scene.control.SplitPane;


abstract class BaseMessageBasedViewController extends BaseConnectionController implements MessageListViewDelegate {

    @FXML
    protected SplitPane splitPane;

    MessageListViewController messageListViewController;

    BaseMessageBasedViewController(String connectionId) {
        super(connectionId);
    }

    void initMessageListView() {
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
