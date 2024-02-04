package org.correomqtt.plugin.save;

import javafx.stage.FileChooser;
import javafx.stage.Stage;
import javafx.stage.Window;
import org.correomqtt.gui.plugin.spi.DetailViewManipulatorHook;
import org.pf4j.Extension;

import java.io.File;
import java.io.IOException;
import java.io.PrintWriter;

@Extension
public class SaveTask implements DetailViewManipulatorHook {

    @Override
    public byte[] manipulate(byte[] bytes) {
        return saveSelection(bytes);
    }

    private byte[] saveSelection(byte[] bytes) {
        FileChooser fileChooser = new FileChooser();
        fileChooser.getExtensionFilters().add(new FileChooser.ExtensionFilter("TXT files (*.txt)", "*.txt"));

        File file = fileChooser.showSaveDialog(Stage.getWindows().stream().filter(Window::isShowing).findFirst().orElseThrow());
        if (file != null) {
            saveTextToFile(new String(bytes), file);
        }
        return bytes;
    }

    private void saveTextToFile(String content, File file) {
        try {
            PrintWriter writer = new PrintWriter(file);
            writer.println(content);
            writer.close();
        } catch (IOException e) {
            e.printStackTrace();
        }
    }
}
