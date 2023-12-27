package org.correomqtt.gui.business;

import lombok.extern.slf4j.Slf4j;
import org.correomqtt.business.model.ConnectionExportDTO;
import org.correomqtt.business.model.ScriptExecutionDTO;
import org.correomqtt.business.services.ConnectService;
import org.correomqtt.business.services.DisconnectService;
import org.correomqtt.business.services.ExportConnectionService;
import org.correomqtt.business.services.ExportMessageService;
import org.correomqtt.business.services.ImportConnectionService;
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
public class MessageTaskFactory {

    private MessageTaskFactory() {
        // private constructor
    }

    public static void publish(String connectionId, MessagePropertiesDTO messagePropertiesDTO) {
        new GuiService<>(new PublishService(connectionId,
                MessageTransformer.propsToDTO(messagePropertiesDTO)),
                PublishService::publish).start();
    }

    private static MessagePropertiesDTO executeOnPublishMessageExtensions(String connectionId, MessagePropertiesDTO messagePropertiesDTO) {
        MessageExtensionDTO messageExtensionDTO = new MessageExtensionDTO(messagePropertiesDTO);
        for (OutgoingMessageHook<?> p : PluginManager.getInstance().getOutgoingMessageHooks()) {
            log.info("Publish {}", p);
            messageExtensionDTO = p.onPublishMessage(connectionId, messageExtensionDTO);
        }
        return MessageTransformer.mergeProps(messageExtensionDTO, messagePropertiesDTO);
    }

    public static void subscribe(String connectionId, SubscriptionPropertiesDTO subscriptionDTO) {
        new GuiService<>(new SubscribeService(connectionId,
                SubscriptionTransformer.propsToDTO(subscriptionDTO)),
                SubscribeService::subscribe).start();
    }

    public static void unsubscribe(String connectionId, SubscriptionPropertiesDTO subscriptionPropertiesDTO) {
        new GuiService<>(new UnsubscribeService(connectionId,
                SubscriptionTransformer.propsToDTO(subscriptionPropertiesDTO)),
                UnsubscribeService::unsubscribe).start();
    }

    public static void connect(String connectionId) {
        new GuiService<>(new ConnectService(connectionId), ConnectService::connect).start();
    }

    public static void disconnect(String connectionId) {
        new GuiService<>(new DisconnectService(connectionId),
                DisconnectService::disconnect).start();
    }

    public static void importMessage(String connectionId, File file) {
        new GuiService<>(new ImportMessageService(connectionId, file), ImportMessageService::importMessage).start();
    }

    public static void exportMessage(String connectionId, File file, MessagePropertiesDTO messageDTO) {
        new GuiService<>(new ExportMessageService(connectionId, file, MessageTransformer.propsToDTO(messageDTO)),
                ExportMessageService::exportMessage).start();

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

    public static void exportConnection(String connectionId, File file, ConnectionExportDTO connectionExportDTO) {
        new GuiService<>(new ExportConnectionService(connectionId, file, connectionExportDTO),
                ExportConnectionService::exportConnection).start();

    }

    public static void importConnection(File file) {
        new GuiService<>(new ImportConnectionService(file),
                ImportConnectionService::importConnection).start();

    }

    public static void reconnect(String connectionId) {
        new GuiService<>(new ConnectService(connectionId), ConnectService::reconnect).start();
    }
}
