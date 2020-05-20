package org.correomqtt.gui.utils;

import org.correomqtt.business.services.SettingsService;
import org.correomqtt.gui.business.TaskFactory;
import org.correomqtt.gui.model.MessagePropertiesDTO;
import javafx.stage.FileChooser;
import javafx.stage.Stage;

import java.io.File;
import java.util.ResourceBundle;

public class MessageUtils {
    private static ResourceBundle resources;

    private MessageUtils() {

    }

    public static void saveMessage(String connectionId, MessagePropertiesDTO messageDTO, Stage stage) {
        resources = ResourceBundle.getBundle("org.correomqtt.i18n", SettingsService.getInstance().getSettings().getCurrentLocale());

        FileChooser fileChooser = new FileChooser();
        fileChooser.setTitle(resources.getString("messageUtilsTitle"));
        FileChooser.ExtensionFilter extFilter = new FileChooser.ExtensionFilter(resources.getString("messageUtilsDescription"), "*.cqm");
        fileChooser.getExtensionFilters().add(extFilter);

        File file = fileChooser.showSaveDialog(stage);
        if (file != null) {
            TaskFactory.exportMessage(connectionId, file, messageDTO);
        }
    }
}
