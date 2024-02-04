package org.correomqtt.plugins.systopic.controller;

import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.geometry.Insets;
import javafx.scene.control.Button;
import javafx.scene.layout.HBox;
import org.correomqtt.core.connection.ConnectionStateChangedEvent;
import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.di.Observes;
import org.correomqtt.di.ObservesFilter;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;

import static org.correomqtt.core.connection.ConnectionState.CONNECTED;
import static org.correomqtt.core.connection.ConnectionState.DISCONNECTED_GRACEFUL;
import static org.correomqtt.core.connection.ConnectionState.DISCONNECTED_UNGRACEFUL;

@DefaultBean
public class SysTopicButtonController {

    private final SysTopicViewControllerFactory sysTopicViewControllerFactory;
    private Logger LOGGER = LoggerFactory.getLogger(SysTopicButtonController.class);

    @FXML
    private Button SYSbutton;

    private String connectionId;




    @Inject
    public SysTopicButtonController(
           SysTopicViewControllerFactory sysTopicViewControllerFactory,
           @Assisted String connectionId) {
        this.connectionId = connectionId;
        this.sysTopicViewControllerFactory = sysTopicViewControllerFactory;

        //TODO cleanup
    }

    public void addItems(HBox toolbar, int indexToinsert) {
        HBox.setMargin(SYSbutton, new Insets(0, 0, 0, 5));
        toolbar.getChildren().add(indexToinsert, SYSbutton);
    }

    @FXML
    void OnClicked() throws IOException {
        LOGGER.info("$SYS Button clicked with connectionId:" + connectionId);
        sysTopicViewControllerFactory.create(connectionId).showAsDialog();
    }

    public void onConnectionChangedEvent(@Observes ConnectionStateChangedEvent event) {
        if (event.getState() == CONNECTED || event.getState() == DISCONNECTED_GRACEFUL || event.getState() == DISCONNECTED_UNGRACEFUL) {
            Platform.runLater(() -> SYSbutton.setDisable(false));
        } else {
            Platform.runLater(() -> SYSbutton.setDisable(true));
        }
    }

    @ObservesFilter(value = "connectionId")
    public String getConnectionId() {
        return connectionId;
    }
}




