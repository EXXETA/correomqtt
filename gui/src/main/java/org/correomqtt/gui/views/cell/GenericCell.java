package org.correomqtt.gui.views.cell;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedInject;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.layout.Pane;
import org.correomqtt.core.settings.SettingsProvider;
import org.correomqtt.core.model.GenericTranslatable;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;

@SuppressWarnings("java:S110")
public class GenericCell<T extends GenericTranslatable> extends ListCell<T> {

    private static final Logger LOGGER = LoggerFactory.getLogger(GenericCell.class);
    private final SettingsProvider settingsProvider;
    private final ListView<T> listView;

    @FXML
    private Pane mainNode;
    @FXML
    private Label label;
    @FXML
    private ResourceBundle resources;

    private FXMLLoader loader;

    @AssistedInject
    public GenericCell(SettingsProvider settingsProvider,
                       @Assisted ListView<T> listView) {
        this.settingsProvider = settingsProvider;
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
                            ResourceBundle.getBundle("org.correomqtt.i18n", settingsProvider.getSettings().getCurrentLocale()));
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
                label.setText(resources.containsKey(translationKey)?resources.getString(translationKey):translationKey);
            }else{
                label.setText(item.toString());
            }
            setText(null);
            setGraphic(mainNode);
        }
    }
}
