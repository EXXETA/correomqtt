package org.correomqtt.gui.utils;

import javafx.stage.FileChooser;
import javafx.stage.Stage;
import org.correomqtt.core.importexport.log.ExportLogTaskFactory;
import org.correomqtt.core.settings.SettingsManager;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.fxmisc.richtext.StyleClassedTextArea;

import java.io.File;
import java.util.ResourceBundle;

@DefaultBean
public class LogAreaUtils {

    private final ExportLogTaskFactory exportLogTaskFactory;

    private final SettingsManager settingsManager;

    @Inject
    public LogAreaUtils(ExportLogTaskFactory exportLogTaskFactory, SettingsManager settingsManager) {
        this.exportLogTaskFactory = exportLogTaskFactory;
        this.settingsManager = settingsManager;
    }

    public static void appendColorful(StyleClassedTextArea area, String msg) {

        if (msg == null) {
            return;
        }

        String[] matches = msg.split("\u001B");
        String cssClass;
        for (String match : matches) {
            String str;
            if (match.startsWith("[36m")) {
                cssClass = "cyan";
                str = match.substring(4);
            } else if (match.startsWith("[34m")) {
                cssClass = "blue";
                str = match.substring(4);
            } else if (match.startsWith("[31m")) {
                cssClass = "orange";
                str = match.substring(4);
            } else if (match.startsWith("[33m")) {
                cssClass = "yellow";
                str = match.substring(4);
            } else if (match.startsWith("[35m")) {
                cssClass = "magenta";
                str = match.substring(4);
            } else if (match.startsWith("[1;31m")) {
                cssClass = "red";
                str = match.substring(6);
            } else if (match.startsWith("[0;39m")) {
                cssClass = "default";
                str = match.substring(6);
            } else {
                cssClass = "default";
                str = match;
            }
            area.append(str, cssClass);
        }
    }

    public void saveLogs(String logs, Stage stage) {
        ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", settingsManager.getSettings().getCurrentLocale());

        FileChooser fileChooser = new FileChooser();
        fileChooser.setTitle(resources.getString("logExportTile"));
        FileChooser.ExtensionFilter extFilter = new FileChooser.ExtensionFilter(resources.getString("logExportDescription"), "*.log");
        fileChooser.getExtensionFilters().add(extFilter);

        File file = fileChooser.showSaveDialog(stage);
        if (file != null) {
            exportLogTaskFactory.create(file, logs).run();
        }
    }
}
