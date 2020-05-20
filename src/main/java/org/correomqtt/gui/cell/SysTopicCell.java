package org.correomqtt.gui.cell;

import org.correomqtt.business.model.SysTopic;
import org.correomqtt.business.services.SettingsService;
import org.correomqtt.gui.model.SysTopicPropertiesDTO;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.layout.Pane;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;

public class SysTopicCell extends ListCell<SysTopicPropertiesDTO> {

    private static final Logger LOGGER = LoggerFactory.getLogger(SysTopicCell.class);

    private final ListView<SysTopicPropertiesDTO> listView;

    @SuppressWarnings("unused")
    @FXML
    private Pane mainNode;

    @SuppressWarnings("unused")
    @FXML
    private Label topicLabel;

    @SuppressWarnings("unused")
    @FXML
    private Label translationLabel;

    @SuppressWarnings("unused")
    @FXML
    private Label descriptionLabel;

    @SuppressWarnings("unused")
    @FXML
    private Label valueLabel;
    @FXML
    private ResourceBundle resources;

    private FXMLLoader loader;

    private SysTopicPropertiesDTO sysTopicDTO;

    public SysTopicCell(ListView<SysTopicPropertiesDTO> listView) {
        this.listView = listView;
    }

    @Override
    protected void updateItem(SysTopicPropertiesDTO sysTopicDTO, boolean empty) {
        super.updateItem(sysTopicDTO, empty);
        if (empty || sysTopicDTO == null) {
            setText(null);
            setGraphic(null);
        } else {

            if (loader == null) {
                try {
                    loader = new FXMLLoader(SysTopicCell.class.getResource("sysTopicCell.fxml"),
                            ResourceBundle.getBundle("org.correomqtt.i18n", SettingsService.getInstance().getSettings().getCurrentLocale()));
                    loader.setController(this);
                    loader.load();

                } catch (Exception e) {
                    LOGGER.error("Exception receiving message:", e);
                    setText(resources.getString("commonRowCreationError"));
                    setGraphic(null);
                    return;
                }

            }
            mainNode.prefWidthProperty().bind(listView.widthProperty().subtract(20));
            setupSysTopic(sysTopicDTO);
            setText(null);
            setGraphic(mainNode);
        }
    }

    private void setupSysTopic(SysTopicPropertiesDTO sysTopicDTO) {
        this.sysTopicDTO = sysTopicDTO;

        SysTopic sysTopic = SysTopic.getSysTopicByTopic(sysTopicDTO.getTopic());
        if (sysTopic == null) {
            topicLabel.setText(sysTopicDTO.getTopic());
            descriptionLabel.setText("");
        } else {
            topicLabel.setText(resources.getString(sysTopic.getTranslation()));
            descriptionLabel.setText(resources.getString(sysTopic.getDescription()));
        }

        valueLabel.setText(sysTopicDTO.getAggregatedPayload());; //TODO UTF-8
    }
}