package org.correomqtt.business.provider;

import org.correomqtt.business.dispatcher.ConfigDispatcher;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.*;

public class BaseUserFileProvider {

    private static final Logger LOGGER = LoggerFactory.getLogger(BaseUserFileProvider.class);

    private static final String OPERATING_SYSTEM = System.getProperty("os.name").toLowerCase();
    private static final String USER_HOME = System.getProperty("user.home");
    private static final String USER_DIR = System.getProperty("user.dir");

    private static final String MAC_APP_FOLDER_NAME = "CorreoMqtt";
    private static final String WIN_APP_FOLDER_NAME = MAC_APP_FOLDER_NAME;
    private static final String LIN_APP_FOLDER_NAME = ".correomqtt";

    private File file;

    private String targetDirectoryPathCache;

    protected File getFile() {
        return file;
    }

    @SuppressWarnings("ResultOfMethodCallIgnored")
    void prepareFile(String id, String filename) throws IOException {

        String targetDirectoryPath = getTargetDirectoryPath();

        if (!new File(targetDirectoryPath).exists() && !new File(targetDirectoryPath).mkdir()) {
            ConfigDispatcher.getInstance().onConfigDirectoryEmpty();
        }

        File targetFile;
        if (id == null) {
            targetFile = new File(targetDirectoryPath + File.separator + filename);
        } else {
            targetFile = new File(targetDirectoryPath + File.separator + id + "_" + filename);
        }

        if (!targetFile.exists()) {
            try (InputStream inputStream = SettingsProvider.class.getResourceAsStream(filename)) {
                if(inputStream != null) {
                    byte[] buffer = new byte[inputStream.available()];
                    if(inputStream.read(buffer) > 0) {
                        try (OutputStream outStream = new FileOutputStream(targetFile)) {
                            outStream.write(buffer);
                        }
                    }
                }else{
                    LOGGER.warn("Can not read file {}", filename);
                }
            }
        }

        this.file = targetFile;
    }

    void prepareFile(String configFileName) throws IOException {
        prepareFile(null, configFileName);
    }

    void saveToUserDirectory(String filename, String content) {

        String targetDirectoryPath = getTargetDirectoryPath();
        if (!new File(targetDirectoryPath).exists() && !new File(targetDirectoryPath).mkdir()) {
            ConfigDispatcher.getInstance().onConfigDirectoryEmpty();
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
                ConfigDispatcher.getInstance().onAppDataNull();
            } else {
                targetDirectoryPathCache = appData + File.separator + WIN_APP_FOLDER_NAME;
            }
        } else if (isMacOS()) {

            if (USER_HOME == null) {
                ConfigDispatcher.getInstance().onUserHomeNull();
            } else {
                targetDirectoryPathCache = USER_HOME + File.separator + "Library" + File.separator + "Application Support" + File.separator + MAC_APP_FOLDER_NAME;
            }
        } else if (isLinux()) {

            if (USER_HOME == null) {
                ConfigDispatcher.getInstance().onUserHomeNull();
            } else {
                targetDirectoryPathCache = USER_HOME + File.separator + LIN_APP_FOLDER_NAME;
            }
        } else {
            LOGGER.warn("User directory can not be found. Using working directory.");
            targetDirectoryPathCache = USER_DIR;
        }

        return targetDirectoryPathCache;
    }
}
