package org.correomqtt.business.fileprovider;

import org.correomqtt.business.eventbus.EventBus;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.BufferedWriter;
import java.io.File;
import java.io.FileOutputStream;
import java.io.FileWriter;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.file.Files;
import java.util.HashMap;
import java.util.Map;

abstract class BaseUserFileProvider {

    private static final Logger LOGGER = LoggerFactory.getLogger(BaseUserFileProvider.class);

    private static final String OPERATING_SYSTEM = System.getProperty("os.name").toLowerCase();
    private static final String USER_HOME = System.getProperty("user.home");
    private static final String USER_DIR = System.getProperty("user.dir");
    private static final String MAC_APP_FOLDER_NAME = "CorreoMqtt";
    private static final String WIN_APP_FOLDER_NAME = MAC_APP_FOLDER_NAME;
    private static final String LIN_APP_FOLDER_NAME = ".correomqtt";

    private static final String LOG_FOLDER_NAME = "logs";

    private static final String SCRIPT_FOLDER_NAME = "scripts";

    private static final String DATA_FOLDER_NAME = "data";
    protected static final String SCRIPT_LOG_FOLDER_NAME = SCRIPT_FOLDER_NAME + File.separator + "logs";

    protected static final String SCRIPT_EXECUTIONS_FOLDER_NAME = SCRIPT_FOLDER_NAME + File.separator + "executions";


    private File file;

    private String targetDirectoryPathCache;

    private Map<String, String> cache = new HashMap<>();

    protected File getFile() {
        return file;
    }

    @SuppressWarnings("ResultOfMethodCallIgnored")
    void prepareFile(String id, String filename) throws IOException {

        String targetDirectoryPath = getTargetDirectoryPath();

        if (!new File(targetDirectoryPath).exists() && !new File(targetDirectoryPath).mkdir()) {
            EventBus.fire(new DirectoryCanNotBeCreatedEvent(targetDirectoryPath));
        }

        File targetFile;
        if (id == null) {
            targetFile = new File(targetDirectoryPath + File.separator + filename);
        } else {
            targetFile = new File(targetDirectoryPath + File.separator + id + "_" + filename);
        }

        if (!targetFile.exists()) {
            try (InputStream inputStream = SettingsProvider.class.getResourceAsStream(filename)) {
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

    void prepareFile(String hookFile) throws IOException {
        prepareFile(null, hookFile);
    }

    void saveToUserDirectory(String filename, String content) {

        String targetDirectoryPath = getTargetDirectoryPath();
        if (!new File(targetDirectoryPath).exists() && !new File(targetDirectoryPath).mkdir()) {
            EventBus.fire(new DirectoryCanNotBeCreatedEvent(targetDirectoryPath));
        }

        try (BufferedWriter writer = new BufferedWriter(new FileWriter(targetDirectoryPath + File.separator + filename))) {
            writer.write(content);
        } catch (IOException e) {
            LOGGER.warn("Error writing file {}", filename, e);
        }
    }

    public boolean isMacOS() {
        return OPERATING_SYSTEM.contains("mac os");
    }

    public boolean isWindows() {
        return OPERATING_SYSTEM.startsWith("windows");
    }

    public boolean isLinux() {
        return OPERATING_SYSTEM.contains("linux")
                || OPERATING_SYSTEM.contains("mpe/ix")
                || OPERATING_SYSTEM.contains("freebsd")
                || OPERATING_SYSTEM.contains("irix")
                || OPERATING_SYSTEM.contains("digital unix")
                || OPERATING_SYSTEM.contains("unix");
    }

    public String getTargetDirectoryPath() {

        if (targetDirectoryPathCache != null) {
            return targetDirectoryPathCache;
        }

        if (isWindows()) {
            String appData = System.getenv("APPDATA");
            if (appData == null) {
                EventBus.fire(new WindowsAppDataNullEvent());
            } else {
                targetDirectoryPathCache = appData + File.separator + WIN_APP_FOLDER_NAME;
            }
        } else if (isMacOS()) {

            if (USER_HOME == null) {
                EventBus.fire(new UserHomeNull());
            } else {
                targetDirectoryPathCache = USER_HOME + File.separator + "Library" + File.separator + "Application Support" + File.separator + MAC_APP_FOLDER_NAME;
            }
        } else if (isLinux()) {

            if (USER_HOME == null) {
                EventBus.fire(new UserHomeNull());
            } else {
                targetDirectoryPathCache = USER_HOME + File.separator + LIN_APP_FOLDER_NAME;
            }
        } else {
            LOGGER.warn("User directory can not be found. Using working directory.");
            targetDirectoryPathCache = USER_DIR;
        }

        return targetDirectoryPathCache;
    }

    public String getLogDirectory() {
        return getFromCache(LOG_FOLDER_NAME);
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


    protected String getFromCache(String dir) {
        return getFromCache(dir, true);
    }

    protected String getFromCache(String dir, boolean autocreate) {
        String path = getTargetDirectoryPath() + File.separator + dir;

        if (!autocreate)
            return path;

        return cache.computeIfAbsent(dir, d -> {
            if (!new File(path).exists() && !new File(path).mkdirs()) {
                EventBus.fire(new DirectoryCanNotBeCreatedEvent(path));
                throw new IllegalStateException("Can not create directory: " + path);
            }
            return path;
        });
    }
}
