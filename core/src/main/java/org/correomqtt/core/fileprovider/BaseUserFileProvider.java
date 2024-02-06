package org.correomqtt.core.fileprovider;

import org.correomqtt.core.utils.DirectoryUtils;
import org.correomqtt.di.SoyEvents;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.BufferedWriter;
import java.io.File;
import java.io.FileOutputStream;
import java.io.FileWriter;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.HashMap;
import java.util.Map;

public abstract class BaseUserFileProvider {

    private static final Logger LOGGER = LoggerFactory.getLogger(BaseUserFileProvider.class);

    private static final String SCRIPT_FOLDER_NAME = "scripts";
    protected static final String SCRIPT_LOG_FOLDER_NAME = SCRIPT_FOLDER_NAME + File.separator + "logs";

    protected static final String SCRIPT_EXECUTIONS_FOLDER_NAME = SCRIPT_FOLDER_NAME + File.separator + "executions";
    private final Map<String, String> cache = new HashMap<>();
    protected final SoyEvents soyEvents;
    private File file;

    protected BaseUserFileProvider(SoyEvents soyEvents){
        this.soyEvents = soyEvents;
    }

    protected File getFile() {
        return file;
    }

    protected void prepareFile(String hookFile) throws IOException {
        prepareFile(null, hookFile);
    }

    @SuppressWarnings("ResultOfMethodCallIgnored")
    void prepareFile(String id, String filename) throws IOException {

        String targetDirectoryPath = DirectoryUtils.getTargetDirectoryPath();

        if (!new File(targetDirectoryPath).exists() && !new File(targetDirectoryPath).mkdir()) {
            soyEvents.fire(new DirectoryCanNotBeCreatedEvent(targetDirectoryPath));
        }

        File targetFile;
        if (id == null) {
            targetFile = new File(targetDirectoryPath + File.separator + filename);
        } else {
            targetFile = new File(targetDirectoryPath + File.separator + id + "_" + filename);
        }

        if (!targetFile.exists()) {
            try (InputStream inputStream = BaseUserFileProvider.class.getResourceAsStream(filename)) {
                if (inputStream != null) {
                    byte[] buffer = new byte[inputStream.available()];
                    if (inputStream.read(buffer) > 0) {
                        try (OutputStream outStream = new FileOutputStream(targetFile)) {
                            outStream.write(buffer);
                        }
                    }
                } else {
                    LOGGER.warn("Can not read file {}", filename);
                }
            }
        }

        this.file = targetFile;
    }

    protected void saveToUserDirectory(String filename, String content) {

        String targetDirectoryPath = DirectoryUtils.getTargetDirectoryPath();
        if (!new File(targetDirectoryPath).exists() && !new File(targetDirectoryPath).mkdir()) {
            soyEvents.fire(new DirectoryCanNotBeCreatedEvent(targetDirectoryPath));
        }

        try (BufferedWriter writer = new BufferedWriter(new FileWriter(targetDirectoryPath + File.separator + filename))) {
            writer.write(content);
        } catch (IOException e) {
            LOGGER.warn("Error writing file {}", filename, e);
        }
    }
    protected String getFromCache(String dir) {
        return getFromCache(dir, true);
    }

    protected String getFromCache(String dir, boolean autocreate) {
        String path = DirectoryUtils.getTargetDirectoryPath() + File.separator + dir;

        if (!autocreate)
            return path;

        return cache.computeIfAbsent(dir, d -> {
            if (!new File(path).exists() && !new File(path).mkdirs()) {
                soyEvents.fire(new DirectoryCanNotBeCreatedEvent(path));
                throw new IllegalStateException("Can not create directory: " + path);
            }
            return path;
        });
    }

    public String getScriptLogDirectory(String filename) {
        return getScriptLogDirectory(filename, true);
    }

    public String getScriptLogDirectory(String filename, boolean autocreate) {
        return getFromCache(SCRIPT_LOG_FOLDER_NAME + File.separator + filename, autocreate);
    }

    public String getScriptExecutionsDirectory(String filename) {
        return getScriptExecutionsDirectory(filename, true);
    }

    public String getScriptExecutionsDirectory(String filename, boolean autocreate) {
        return getFromCache(SCRIPT_EXECUTIONS_FOLDER_NAME + File.separator + filename, autocreate);
    }
}
