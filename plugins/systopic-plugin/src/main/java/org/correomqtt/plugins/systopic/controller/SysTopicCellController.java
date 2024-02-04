package org.correomqtt.plugins.systopic.controller;

import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.layout.Pane;
import org.correomqtt.core.CoreManager;
import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.plugins.systopic.model.SysTopic;
import org.correomqtt.plugins.systopic.model.SysTopicPropertiesDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;

@DefaultBean
public class SysTopicCellController extends ListCell<SysTopicPropertiesDTO> {

    private static final Logger LOGGER = LoggerFactory.getLogger(SysTopicCellController.class);

    private final ListView<SysTopicPropertiesDTO> listView;
    private final CoreManager coreManager;
    @FXML
    private Pane mainNode;
    @FXML
    private Label topicLabel;
    @FXML
    private Label translationLabel;
    @FXML
    private Label descriptionLabel;
    @FXML
    private Label valueLabel;
    @FXML
    private ResourceBundle resources;

    private FXMLLoader loader;

    private SysTopicPropertiesDTO sysTopicDTO;




    @Inject
    public SysTopicCellController(CoreManager coreManager,
                                  @Assisted ListView<SysTopicPropertiesDTO> listView) {
        this.coreManager = coreManager;
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
                    loader = new FXMLLoader(SysTopicCellController.class.getResource("/org/correomqtt/plugins/systopic/cell/sysTopicCell.fxml"),
                            ResourceBundle.getBundle("org.correomqtt.plugins.systopic.i18n", coreManager.getSettingsManager().getSettings().getCurrentLocale()));
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

        valueLabel.setText(sysTopicDTO.getAggregatedPayload());
        ; //TODO UTF-8
    }
}