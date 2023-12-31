package org.correomqtt.gui.utils;

import javafx.stage.FileChooser;
import javafx.stage.Stage;
import org.correomqtt.business.fileprovider.SettingsProvider;
import org.correomqtt.business.importexport.messages.ExportMessageTask;
import org.correomqtt.gui.model.MessagePropertiesDTO;
import org.correomqtt.gui.transformer.MessageTransformer;

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
            new ExportMessageTask(connectionId, file, MessageTransformer.propsToDTO(messageDTO)).run();;
        }
    }
}
