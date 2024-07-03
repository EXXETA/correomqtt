package org.correomqtt.core.fileprovider;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.apache.commons.io.FileUtils;
import org.correomqtt.core.utils.DirectoryUtils;
import org.correomqtt.di.SoyEvents;
import org.correomqtt.core.model.HooksDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import org.correomqtt.di.Inject;
import org.correomqtt.di.SingletonBean;
import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.InvalidPathException;
import java.util.Iterator;
import java.util.List;

@SingletonBean
public class PluginConfigProvider extends BaseUserFileProvider {

    private static final Logger LOGGER = LoggerFactory.getLogger(PluginConfigProvider.class);

    private static final String HOOK_FILE_NAME = "hooks.json";
    private static final String EX_MSG_PREPARE_PLUGIN_FOLDER = "Could not create plugin folder.";
    private static final String PLUGIN_FOLDER = "plugins";

    private HooksDTO hooksDTO;
    private String pluginPath;

    @Inject
    public PluginConfigProvider(SoyEvents soyEvents) {
        super(soyEvents);

        try {
            prepareFile(HOOK_FILE_NAME);
        } catch (InvalidPathException | SecurityException | UnsupportedOperationException | IOException e) {
            LOGGER.error("Error writing hook file {}. ", HOOK_FILE_NAME, e);
            soyEvents.fire(new UnaccessibleHookFileEvent(e));
        }

        preparePluginPath();

        try {
            hooksDTO = new ObjectMapper().readValue(getFile(), HooksDTO.class);
        } catch (IOException e) {
            LOGGER.error("Exception parsing hooks file {}", HOOK_FILE_NAME, e);
            soyEvents.fire(new InvalidHooksFileEvent(e));
        }

    }

    public List<HooksDTO.Extension> getOutgoingMessageHooks() {
        return hooksDTO.getOutgoingMessages();
    }

    public List<HooksDTO.Extension> getIncomingMessageHooks() {
        return hooksDTO.getIncomingMessages();
    }

    public List<HooksDTO.DetailViewTask> getDetailViewTasks() {
        return hooksDTO.getDetailViewTasks();
    }

    public List<HooksDTO.MessageValidator> getMessageValidators() {
        return hooksDTO.getMessageValidators();
    }

    private void preparePluginPath() {
        pluginPath = DirectoryUtils.getTargetDirectoryPath() + File.separator + PLUGIN_FOLDER;
        File pluginFolder = new File(pluginPath);
        if (!pluginFolder.exists() && !pluginFolder.mkdir()) {
            LOGGER.error(EX_MSG_PREPARE_PLUGIN_FOLDER);
        }
    }

    public boolean migrationRequired() {
        File oldJarFolder = new File(pluginPath + File.separator + "jars");
        File oldConfigFolder = new File(pluginPath + File.separator + "config");
        File oldProtocolXml = new File(pluginPath + File.separator + "protocol.xml");

        return oldJarFolder.exists() || oldConfigFolder.exists() || oldProtocolXml.exists();
    }

    public void migratePluginFolder() {
        File pluginFolder = new File(pluginPath);
        File oldJarFolder = new File(pluginPath + File.separator + "jars");
        File oldConfigFolder = new File(pluginPath + File.separator + "config");
        File oldProtocolXml = new File(pluginPath + File.separator + "protocol.xml");

        if (oldJarFolder.exists()) {
            try {
                Iterator<File> iterator = FileUtils.iterateFiles(oldJarFolder, new String[]{"jar"}, false);
                while (iterator.hasNext()) {
                    FileUtils.copyFileToDirectory(iterator.next(), pluginFolder);
                }
                FileUtils.deleteDirectory(oldJarFolder);
            } catch (IOException e) {
                LOGGER.error("Unable to migrate jars folder. Skip.");
            }
        }

        if (oldConfigFolder.exists()) {
            try {
                FileUtils.deleteDirectory(oldConfigFolder);
            } catch (IOException e) {
                LOGGER.error("Unable to delete plugin config folder. Skip.");
            }
        }

        if (oldProtocolXml.exists()) {
            try {
                Files.delete(oldProtocolXml.toPath());
            } catch (IOException e) {
                LOGGER.error("Unable to delete obsolete protocol.xml. Skip.");
            }
        }
    }


    public String getPluginPath() {
        return pluginPath;
    }
    public String getClassPath(){
        return getClass().getProtectionDomain().getCodeSource().getLocation().getPath();
    }

}
