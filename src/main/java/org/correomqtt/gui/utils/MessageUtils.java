package org.correomqtt.gui.utils;

import com.exxeta.correomqtt.business.services.ConfigService;
import com.exxeta.correomqtt.gui.business.TaskFactory;
import com.exxeta.correomqtt.gui.model.MessagePropertiesDTO;
import javafx.stage.FileChooser;
import javafx.stage.Stage;

import java.io.File;
import java.util.ResourceBundle;

public class MessageUtils {
    private static ResourceBundle resources;

    private MessageUtils() {

    }

    public static void saveMessage(String connectionId, MessagePropertiesDTO messageDTO, Stage stage) {
        resources = ResourceBundle.getBundle("com.exxeta.correomqtt.i18n", ConfigService.getInstance().getSettings().getCurrentLocale());

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
