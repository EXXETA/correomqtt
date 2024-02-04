package org.correomqtt.plugins.systopic;

import dagger.Component;
import javafx.scene.layout.HBox;
import org.correomqtt.MainComponent;
import org.correomqtt.gui.plugin.ExtensionComponent;
import org.correomqtt.gui.plugin.spi.MainToolbarHook;
import org.correomqtt.plugins.systopic.controller.SysTopicButtonController;
import org.pf4j.Extension;

import org.correomqtt.core.cdi.Inject;
import org.correomqtt.core.cdi.SingletonBean;

@Extension
public class SysTopicExtension implements MainToolbarHook {


    private final SysTopicButtonControllerFactory sysTopicButtonControllerFactory;

    @SingletonBean
    @Component(dependencies = MainComponent.class)
    public interface Factory extends ExtensionComponent<SysTopicExtension> {

    }

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
