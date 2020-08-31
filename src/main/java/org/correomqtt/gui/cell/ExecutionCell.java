package org.correomqtt.gui.cell;

import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.layout.Pane;
import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.gui.model.ExecutionPropertiesDTO;
import org.correomqtt.gui.model.ScriptingPropertiesDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;

public class ExecutionCell extends ListCell<ExecutionPropertiesDTO> {

    private static final Logger LOGGER = LoggerFactory.getLogger(ExecutionCell.class);
    private final ListView<ExecutionPropertiesDTO> listView;

    @FXML
    private Pane mainNode;
    @FXML
    private Label nameLabel;
    @FXML
    private ResourceBundle resources;

    private FXMLLoader loader;

    @FXML
    public void initialize() {
        mainNode.getStyleClass().add(SettingsProvider.getInstance().getIconModeCssClass());
    }

    public ExecutionCell(ListView<ExecutionPropertiesDTO> listView) {
        this.listView = listView;
    }

    @Override
    protected void updateItem(ExecutionPropertiesDTO executionDTO, boolean empty) {
        super.updateItem(executionDTO, empty);

        if (empty || executionDTO == null) {
            setText(null);
            setGraphic(null);
        } else {

            if (loader == null) {
                try {
                    loader = new FXMLLoader(SubscriptionViewCell.class.getResource("scriptCell.fxml"),
                            ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale()));
                    loader.setController(this);
                    loader.load();

                } catch (Exception e) {
                    LOGGER.error("Exception rendering script:", e);
                    setText(resources.getString("commonRowCreationError"));
                    setGraphic(null);
                    return;
                }

            }
            mainNode.prefWidthProperty().bind(listView.widthProperty().subtract(20));
            setScript(executionDTO);
            setText(null);
            setGraphic(mainNode);
        }
    }

    private void setScript(ExecutionPropertiesDTO scriptingDTO) {

        nameLabel.setText(scriptingDTO.getExecutionId()); //TODO
    }
}
