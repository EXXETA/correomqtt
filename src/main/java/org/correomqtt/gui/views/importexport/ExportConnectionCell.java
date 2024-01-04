package org.correomqtt.gui.views.importexport;

import javafx.beans.property.BooleanProperty;
import javafx.beans.value.ObservableValue;
import javafx.css.PseudoClass;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.scene.control.CheckBox;
import javafx.scene.control.Label;
import javafx.scene.control.cell.CheckBoxListCell;
import javafx.scene.layout.AnchorPane;
import javafx.util.Callback;
import org.controlsfx.control.CheckListView;
import org.correomqtt.business.fileprovider.SettingsProvider;
import org.correomqtt.gui.views.connections.SubscriptionViewCell;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;

public class ExportConnectionCell extends CheckBoxListCell<ConnectionPropertiesDTO> {
    private static final Logger LOGGER = LoggerFactory.getLogger(ExportConnectionCell.class);
    private final CheckListView<ConnectionPropertiesDTO> listView;

    @FXML
    private Label nameLabel;
    @FXML
    private Label descriptionLabel;
    @FXML
    private CheckBox checkbox;
    @FXML
    public AnchorPane mainNode;
    @FXML
    private ResourceBundle resources;
    private FXMLLoader loader;
    private ObservableValue<Boolean> booleanProperty;

    public ExportConnectionCell(CheckListView<ConnectionPropertiesDTO> listView) {
        super(listView::getItemBooleanProperty);
        this.listView = listView;
    }

    @Override
    public void updateItem(ConnectionPropertiesDTO connectionDTO, boolean empty) {
        super.updateItem(connectionDTO, empty);
        if (empty || connectionDTO == null) {
            setText(null);
            setGraphic(null);
        } else {

            if (loader == null) {
                try {
                    loader = new FXMLLoader(SubscriptionViewCell.class.getResource("exportConnectionCell.fxml"),
                            ResourceBundle.getBundle("org.correomqtt.i18n",
                                    SettingsProvider.getInstance().getSettings().getCurrentLocale()));
                    loader.setController(this);
                    loader.load();

                } catch (Exception e) {
                    LOGGER.error("Exception rendering connection:", e);
                    setText(null);
                    setGraphic(null);
                    return;
                }

            }
            mainNode.prefWidthProperty().bind(listView.widthProperty().subtract(20));
            setConnection(connectionDTO);
            setText(null);
            setGraphic(mainNode);

            Callback<ConnectionPropertiesDTO, ObservableValue<Boolean>> callback = getSelectedStateCallback();
            if (booleanProperty != null) {
                checkbox.selectedProperty().unbindBidirectional((BooleanProperty) booleanProperty);
            }
            booleanProperty = callback.call(connectionDTO);
            if (booleanProperty != null) {
                checkbox.selectedProperty().bindBidirectional((BooleanProperty) booleanProperty);
            }

            this.disabledProperty().addListener((observableValue, oldValue, newValue) -> processDisabled(newValue));

            setOnMouseClicked(e -> onMouseClicked());
            setOnMouseExited(e -> setHoverPseudoClass(false));
            setOnMouseEntered(e -> setHoverPseudoClass(true));
        }
    }

    private void onMouseClicked() {
        BooleanProperty checked = listView.getItemBooleanProperty(getItem());
        checked.set(!checked.get());
    }

    private void setHoverPseudoClass(boolean hover) {
        if(checkbox != null) {
            checkbox.pseudoClassStateChanged(PseudoClass.getPseudoClass("hover"),  hover);
        }
    }

    private void setConnection(ConnectionPropertiesDTO connectionDTO) {
        nameLabel.setText(connectionDTO.getName());
        descriptionLabel.setText(connectionDTO.getHostAndPort());
        this.processDisabled(this.isDisabled());
    }

    private void processDisabled(boolean disabled) {

        if (nameLabel == null || descriptionLabel == null)
            return;

        if (Boolean.TRUE.equals(disabled)) {
            descriptionLabel.setText(resources.getString("connectionExportConnectionAlreadyExists"));
        }
    }
}
