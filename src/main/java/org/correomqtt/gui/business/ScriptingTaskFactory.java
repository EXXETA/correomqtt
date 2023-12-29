package org.correomqtt.gui.business;

import lombok.extern.slf4j.Slf4j;
import org.correomqtt.business.model.ScriptExecutionDTO;
import org.correomqtt.business.services.ConnectService;
import org.correomqtt.business.services.DisconnectService;
import org.correomqtt.business.services.ExportMessageService;
import org.correomqtt.business.services.ImportMessageService;
import org.correomqtt.business.services.PublishService;
import org.correomqtt.business.services.ScriptCancelService;
import org.correomqtt.business.services.ScriptLoadService;
import org.correomqtt.business.services.ScriptSubmitService;
import org.correomqtt.business.services.SubscribeService;
import org.correomqtt.business.services.UnsubscribeService;
import org.correomqtt.gui.model.MessagePropertiesDTO;
import org.correomqtt.gui.model.ScriptingPropertiesDTO;
import org.correomqtt.gui.model.SubscriptionPropertiesDTO;
import org.correomqtt.gui.transformer.MessageTransformer;
import org.correomqtt.gui.transformer.ScriptingTransformer;
import org.correomqtt.gui.transformer.SubscriptionTransformer;
import org.correomqtt.plugin.manager.PluginManager;
import org.correomqtt.plugin.model.MessageExtensionDTO;
import org.correomqtt.plugin.spi.OutgoingMessageHook;

import java.io.File;

@Slf4j
public class ScriptingTaskFactory {

    private ScriptingTaskFactory() {
        // private constructor
    }

    public static void loadScript(ScriptingPropertiesDTO scriptingPropertiesDTO) {
        new GuiService<>(new ScriptLoadService(ScriptingTransformer.propsToDTO(scriptingPropertiesDTO)),
                ScriptLoadService::loadScript).start();
    }

    public static void submitScript(ScriptExecutionDTO scriptExecutionDTO) {
        new GuiService<>(new ScriptSubmitService(scriptExecutionDTO),
                ScriptSubmitService::submitScript).start();
    }

    public static void cancelScript(ScriptExecutionDTO scriptExecutionDTO) {
        new GuiService<>(new ScriptCancelService(scriptExecutionDTO),
                ScriptCancelService::cancelScript).start();
    }

    public static void createScript(String filename) {
        new GuiService<>(new ScriptCreateService(filename),
                ScriptCreateService::createScript).start();
    }
}
