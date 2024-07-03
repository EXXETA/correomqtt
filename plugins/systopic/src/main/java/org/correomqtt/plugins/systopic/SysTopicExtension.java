package org.correomqtt.plugins.systopic;

import javafx.scene.layout.HBox;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.gui.plugin.spi.MainToolbarHook;
import org.correomqtt.plugins.systopic.controller.SysTopicButtonController;
import org.correomqtt.plugins.systopic.controller.SysTopicButtonControllerFactory;
import org.pf4j.Extension;

@DefaultBean
@Extension
public class SysTopicExtension implements MainToolbarHook {

    private final SysTopicButtonControllerFactory sysTopicButtonControllerFactory;


    @Inject
    public SysTopicExtension(SysTopicButtonControllerFactory sysTopicButtonControllerFactory) {

        this.sysTopicButtonControllerFactory = sysTopicButtonControllerFactory;
    }

    @Override
    public void onInstantiateMainToolbar(String connectionId, HBox controllViewButtonHBox, int indexToInsert) {
        SysTopicButtonController controller = sysTopicButtonControllerFactory.create(connectionId);
        SysTopicPlugin.loadFXML("/org/correomqtt/plugins/systopic/controller/sysTopicButton.fxml", controller);
        controller.addItems(controllViewButtonHBox, indexToInsert);
    }
}
