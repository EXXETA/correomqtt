package org.correomqtt.gui.utils;

import javafx.stage.FileChooser;
import javafx.stage.Stage;
import org.correomqtt.core.importexport.messages.ExportMessageTaskFactory;
import org.correomqtt.core.settings.SettingsManager;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.gui.model.MessagePropertiesDTO;
import org.correomqtt.gui.transformer.MessageTransformer;

import java.io.File;
import java.util.ResourceBundle;

@DefaultBean
public class MessageUtils {

    private final SettingsManager settingsManager;
    private final ExportMessageTaskFactory exportMessageTaskFactory;

    @Inject
    MessageUtils(ExportMessageTaskFactory exportMessageTaskFactory,
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
