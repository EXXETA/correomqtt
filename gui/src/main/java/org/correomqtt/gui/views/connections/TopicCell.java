package org.correomqtt.gui.views.connections;

import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.layout.Pane;
import org.correomqtt.core.settings.SettingsManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;

@SuppressWarnings("java:S110")
@DefaultBean
public class TopicCell extends ListCell<String> {

    private static final Logger LOGGER = LoggerFactory.getLogger(TopicCell.class);
    private final SettingsManager settingsManager;
    private final ListView<String> listView;

    @FXML
    private Pane mainNode;
    @FXML
    private Label topicLabel;
    @FXML
    private ResourceBundle resources;

    private FXMLLoader loader;


    @Inject
    public TopicCell(SettingsManager settingsManager,
                     @Assisted ListView<String> listView) {
        this.settingsManager = settingsManager;
        this.listView = listView;
    }

    @Override
    protected void updateItem(String topic, boolean empty) {
        super.updateItem(topic, empty);

        if (empty || topic == null) {
            setText(null);
            setGraphic(null);
        } else {

            if (loader == null) {
                try {
                    loader = new FXMLLoader(TopicCell.class.getResource("topicCell.fxml"),
                            ResourceBundle.getBundle("org.correomqtt.i18n", settingsManager.getSettings().getCurrentLocale()));
                    loader.setController(this);
                    loader.load();

                } catch (Exception e) {
                    LOGGER.error("Exception rendering topic cell:", e);
                    setText(resources.getString("commonRowCreationError"));
                    setGraphic(null);
                    return;
                }

            }
            mainNode.prefWidthProperty().bind(listView.widthProperty().subtract(20));
            setTopic(topic);
            setText(null);
            setGraphic(mainNode);
        }
    }

    private void setTopic(String topic) {
        topicLabel.setText(topic);
    }
}
