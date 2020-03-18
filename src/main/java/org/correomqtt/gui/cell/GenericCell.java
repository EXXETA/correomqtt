package org.correomqtt.gui.cell;

import org.correomqtt.business.services.ConfigService;
import org.correomqtt.gui.model.GenericCellModel;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.layout.Pane;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;

public class GenericCell<T extends GenericCellModel> extends ListCell<T> {

    private static final Logger LOGGER = LoggerFactory.getLogger(GenericCell.class);
    private final ListView<T> listView;

    @FXML
    private Pane mainNode;
    @FXML
    private Label label;
    @FXML
    private ResourceBundle resources;

    private FXMLLoader loader;

    public GenericCell(ListView<T> listView) {
        this.listView = listView;
    }

    @Override
    protected void updateItem(T item, boolean empty) {
        super.updateItem(item, empty);

        if (empty || item == null) {
            setText(null);
            setGraphic(null);
        } else {

            if (loader == null) {
                try {
                    loader = new FXMLLoader(GenericCell.class.getResource("genericCell.fxml"),
                            ResourceBundle.getBundle("org.correomqtt.i18n", ConfigService.getInstance().getSettings().getCurrentLocale()));
                    loader.setController(this);
                    loader.load();

                } catch (Exception e) {
                    LOGGER.error("Exception rendering generic cell:", e);
                    setText(resources.getString("commonRowCreationError"));
                    setGraphic(null);
                    return;
                }

            }
            mainNode.prefWidthProperty().bind(listView.widthProperty().subtract(20));

            String translationKey = item.getLabelTranslationKey();
            if(translationKey != null) {
                label.setText(resources.getString(translationKey));
            }else{
                label.setText(item.toString());
            }
            setText(null);
            setGraphic(mainNode);
        }
    }
}
