package com.exxeta.correomqtt.gui.controller;

import javafx.fxml.FXML;
import javafx.scene.control.SplitPane;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

abstract class BaseMessageBasedViewController extends BaseConnectionController implements MessageListViewDelegate {

    private static final Logger LOGGER = LoggerFactory.getLogger(BaseMessageBasedViewController.class);

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

    @Override
    public void setConnectionId(String connectionId) {
        super.setConnectionId(connectionId);
        messageListViewController.setConnectionId(connectionId);
    }

}
