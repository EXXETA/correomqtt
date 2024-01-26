package org.correomqtt.gui.utils;

import javafx.stage.FileChooser;
import javafx.stage.Stage;
import org.correomqtt.core.importexport.messages.ExportMessageTask;
import org.correomqtt.core.settings.SettingsManager;
import org.correomqtt.gui.model.MessagePropertiesDTO;
import org.correomqtt.gui.transformer.MessageTransformer;

import javax.inject.Inject;
import java.io.File;
import java.util.ResourceBundle;

public class MessageUtils {

    private final SettingsManager settingsManager;
    private final ExportMessageTask.Factory exportMessageTaskFactory;

    @Inject
    MessageUtils(ExportMessageTask.Factory exportMessageTaskFactory,
                 SettingsManager settingsManager) {
        this.exportMessageTaskFactory = exportMessageTaskFactory;
        this.settingsManager = settingsManager;
    }

    public void saveMessage(MessagePropertiesDTO messageDTO, Stage stage) {
        ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", settingsManager.getSettings().getCurrentLocale());

        FileChooser fileChooser = new FileChooser();
        fileChooser.setTitle(resources.getString("messageUtilsTitle"));
        FileChooser.ExtensionFilter extFilter = new FileChooser.ExtensionFilter(resources.getString("messageUtilsDescription"), "*.cqm");
        fileChooser.getExtensionFilters().add(extFilter);

        File file = fileChooser.showSaveDialog(stage);
        if (file != null) {
            exportMessageTaskFactory.create(file, MessageTransformer.propsToDTO(messageDTO)).run();
        }
    }
}
