package com.exxeta.correomqtt.gui.business;

import com.exxeta.correomqtt.business.services.*;
import com.exxeta.correomqtt.gui.model.MessagePropertiesDTO;
import com.exxeta.correomqtt.gui.model.SubscriptionPropertiesDTO;
import com.exxeta.correomqtt.gui.transformer.MessageTransformer;
import com.exxeta.correomqtt.gui.transformer.SubscriptionTransformer;
import com.exxeta.correomqtt.plugin.manager.PluginSystem;
import com.exxeta.correomqtt.plugin.model.MessageExtensionDTO;
import com.exxeta.correomqtt.plugin.spi.PublishMessageHook;

import java.io.File;

public class TaskFactory {

    private TaskFactory() {
        // private constructor
    }

    public static void publish(String connectionId, MessagePropertiesDTO messagePropertiesDTO) {
        messagePropertiesDTO = executeOnPublishMessageExtensions(connectionId, messagePropertiesDTO);

        new GuiService<>(new PublishService(connectionId,
                                            MessageTransformer.propsToDTO(messagePropertiesDTO)),
                         PublishService::publish).start();
    }

    private static MessagePropertiesDTO executeOnPublishMessageExtensions(String connectionId, MessagePropertiesDTO messagePropertiesDTO) {
        MessageExtensionDTO messageExtensionDTO = new MessageExtensionDTO(messagePropertiesDTO);
        for (PublishMessageHook p : PluginSystem.getInstance().getExtensions(PublishMessageHook.class)) {
            messageExtensionDTO = p.onPublishMessage(connectionId, messageExtensionDTO);
        }
        return messageExtensionDTO.merge(messagePropertiesDTO);
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
}
