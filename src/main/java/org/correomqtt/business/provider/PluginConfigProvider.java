package org.correomqtt.business.provider;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.apache.commons.io.FileUtils;
import org.correomqtt.business.dispatcher.ConfigDispatcher;
import org.correomqtt.business.model.HooksDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.io.IOException;
import java.nio.file.AccessDeniedException;
import java.nio.file.DirectoryNotEmptyException;
import java.nio.file.FileAlreadyExistsException;
import java.nio.file.InvalidPathException;
import java.util.Iterator;
import java.util.List;

public class PluginConfigProvider extends BaseUserFileProvider {

    private static final Logger LOGGER = LoggerFactory.getLogger(PluginConfigProvider.class);

    private static final String HOOK_FILE_NAME = "hooks.json";
    private static final String EX_MSG_PREPARE_HOOK = "Exception loading hooks file.";
    private static final String EX_MSG_PREPARE_PLUGIN_FOLDER = "Could not create plugin folder.";
    private static final String PLUGIN_FOLDER = "plugins";

    private static PluginConfigProvider instance = null;

    private HooksDTO hooksDTO;
    private String pluginPath;

    private PluginConfigProvider() {

        try {
            prepareFile(HOOK_FILE_NAME);
        } catch (InvalidPathException e) {
            LOGGER.error(EX_MSG_PREPARE_HOOK, e);
            ConfigDispatcher.getInstance().onInvalidPath();
        } catch (FileAlreadyExistsException e) {
            LOGGER.error(EX_MSG_PREPARE_HOOK, e);
            ConfigDispatcher.getInstance().onFileAlreadyExists();
        } catch (DirectoryNotEmptyException e) {
            LOGGER.error(EX_MSG_PREPARE_HOOK, e);
            ConfigDispatcher.getInstance().onConfigDirectoryEmpty();
        } catch (SecurityException | AccessDeniedException e) {
            LOGGER.error(EX_MSG_PREPARE_HOOK, e);
            ConfigDispatcher.getInstance().onConfigDirectoryNotAccessible();
        } catch (UnsupportedOperationException | IOException e) {
            LOGGER.error(EX_MSG_PREPARE_HOOK, e);
            ConfigDispatcher.getInstance().onConfigPrepareFailure();
        }

        preparePluginPath();

        try {
            hooksDTO = new ObjectMapper().readValue(getFile(), HooksDTO.class);
        } catch (IOException e) {
            LOGGER.error("Exception parsing hooks file {}", HOOK_FILE_NAME, e);
            ConfigDispatcher.getInstance().onInvalidJsonFormat();
        }

    }

    public static synchronized PluginConfigProvider getInstance() {
        if (instance == null) {
            instance = new PluginConfigProvider();
            return instance;
        } else {
            return instance;
        }
    }

    public List<HooksDTO.Extension> getOutgoingMessageHooks(){
        return hooksDTO.getOutgoingMessages();
    }

    public List<HooksDTO.Extension> getIncomingMessageHooks(){
        return hooksDTO.getIncomingMessages();
    }

    public List<HooksDTO.DetailViewTask> getDetailViewTasks(){
        return hooksDTO.getDetailViewTasks();
    }

    public List<HooksDTO.MessageValidator> getMessageValidators(){
        return hooksDTO.getMessageValidators();
    }

    private void preparePluginPath() {
        pluginPath = getTargetDirectoryPath() + File.separator + PLUGIN_FOLDER;
        File pluginFolder = new File(pluginPath);
        if (!pluginFolder.exists() && !pluginFolder.mkdir()) {
            LOGGER.error(EX_MSG_PREPARE_PLUGIN_FOLDER);
        }

        migrateDeprectedJarFolder(pluginFolder);
    }

    private void migrateDeprectedJarFolder(File pluginFolder) {
        File oldJarFolder = new File(pluginPath + File.separator + "jars");
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
    }

    public String getPluginPath() {
        return pluginPath;
    }

}
