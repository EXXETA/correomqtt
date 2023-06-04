package org.correomqtt.gui.utils;

import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.gui.business.MessageTaskFactory;
import org.correomqtt.gui.model.MessagePropertiesDTO;
import javafx.stage.FileChooser;
import javafx.stage.Stage;

import java.io.File;
import java.util.ResourceBundle;

public class MessageUtils {

    private MessageUtils() {
        // private constructor
    }

    public static void saveMessage(String connectionId, MessagePropertiesDTO messageDTO, Stage stage) {
        ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale());

        FileChooser fileChooser = new FileChooser();
        fileChooser.setTitle(resources.getString("messageUtilsTitle"));
        FileChooser.ExtensionFilter extFilter = new FileChooser.ExtensionFilter(resources.getString("messageUtilsDescription"), "*.cqm");
        fileChooser.getExtensionFilters().add(extFilter);

        File file = fileChooser.showSaveDialog(stage);
        if (file != null) {
            MessageTaskFactory.exportMessage(connectionId, file, messageDTO);
        }
    }
}
