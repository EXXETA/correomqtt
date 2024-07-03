package org.correomqtt.plugins.systopic;

import javafx.fxml.FXMLLoader;
import javafx.util.FXPermission;
import org.pf4j.Plugin;
import org.pf4j.PluginRuntimeException;

import java.io.FilePermission;
import java.io.IOException;
import java.security.Permissions;
import java.util.PropertyPermission;

public class SysTopicPlugin extends Plugin {

    public SysTopicPlugin() {
        super();
    }

    static void loadFXML(String resourceName, Object controller) {
        FXMLLoader loader = new FXMLLoader(controller.getClass().getResource(resourceName));
        loader.setController(controller);
        try {
            loader.load();
        } catch (IOException e) {
            throw new PluginRuntimeException("Failed to load layout file");
        }
    }
}
