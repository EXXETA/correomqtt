package org.correomqtt.gui.cell;

import org.correomqtt.business.provider.SettingsProvider;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.layout.Pane;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;

public class TopicCell extends ListCell<String> {

    private static final Logger LOGGER = LoggerFactory.getLogger(TopicCell.class);
    private final ListView<String> listView;

    @FXML
    private Pane mainNode;
    @FXML
    private Label topicLabel;
    @FXML
    private ResourceBundle resources;

    private FXMLLoader loader;

    public TopicCell(ListView<String> listView) {
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
                            ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale()));
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
