package org.correomqtt.gui.cell;

import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.layout.Pane;
import org.correomqtt.business.fileprovider.SettingsProvider;
import org.correomqtt.gui.model.PluginInfoPropertiesDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;

@SuppressWarnings("java:S110")
public class PluginCell extends ListCell<PluginInfoPropertiesDTO> {

    private static final Logger LOGGER = LoggerFactory.getLogger(PluginCell.class);

    @FXML
    Pane mainNode;
    @FXML
    Label nameLabel;
    @FXML
    Label descriptionLabel;
    @FXML
    Label installedLabel;
    @FXML
    Label disabledLabel;
    @FXML
    Label upgradeableLabel;
    @FXML
    Label bundledLabel;
    @FXML
    ResourceBundle resources;

    private final ListView<PluginInfoPropertiesDTO> listView;

    private FXMLLoader loader;

    @FXML
    public void initialize() {
        mainNode.getStyleClass().add(SettingsProvider.getInstance().getIconModeCssClass());
    }

    public PluginCell(ListView<PluginInfoPropertiesDTO> listView) {
        this.listView = listView;
    }

    @Override
    protected void updateItem(PluginInfoPropertiesDTO pluginInfoPropertiesDTO, boolean empty) {
        super.updateItem(pluginInfoPropertiesDTO, empty);

        if (empty || pluginInfoPropertiesDTO == null) {
            setText(null);
            setGraphic(null);
        } else {

            if (loader == null) {
                try {
                    loader = new FXMLLoader(PluginCell.class.getResource("pluginCell.fxml"),
                            ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale()));
                    loader.setController(this);
                    loader.load();

                } catch (Exception e) {
                    LOGGER.error("Exception rendering plugin:", e);
                    setText(resources.getString("commonRowCreationError"));
                    setGraphic(null);
                    return;
                }

            }
            mainNode.prefWidthProperty().bind(listView.widthProperty().subtract(20));
            setPluginInfo(pluginInfoPropertiesDTO);
            setText(null);
            setGraphic(mainNode);
        }
    }

    private void setPluginInfo(PluginInfoPropertiesDTO pluginInfoPropertiesDTO) {
        if (pluginInfoPropertiesDTO.getRepositoryId() == null) {
            nameLabel.setText(pluginInfoPropertiesDTO.getName() + " " + pluginInfoPropertiesDTO.getInstalledVersion());
        } else {
            nameLabel.setText(pluginInfoPropertiesDTO.getName());
        }
        descriptionLabel.setText(pluginInfoPropertiesDTO.getDescription());
        if (pluginInfoPropertiesDTO.getRepositoryId() == null) {
            boolean disabled = Boolean.TRUE.equals(pluginInfoPropertiesDTO.getDisabled());
            upgradeableLabel.setVisible(false);
            upgradeableLabel.setManaged(false);
            installedLabel.setVisible(false);
            installedLabel.setManaged(false);
            disabledLabel.setVisible(disabled);
            disabledLabel.setManaged(disabled);
        } else {
            boolean upgradeable = pluginInfoPropertiesDTO.getUpgradeable();
            upgradeableLabel.setVisible(upgradeable);
            upgradeableLabel.setManaged(upgradeable);
            boolean installed = pluginInfoPropertiesDTO.getInstalledVersion() != null;
            installedLabel.setVisible(installed);
            installedLabel.setManaged(installed);
            disabledLabel.setVisible(false);
            disabledLabel.setManaged(false);
        }

        if (Boolean.TRUE.equals(pluginInfoPropertiesDTO.getBundled())) {
            bundledLabel.setVisible(true);
            bundledLabel.setManaged(true);
        } else {
            bundledLabel.setVisible(false);
            bundledLabel.setManaged(false);
        }
    }

}
