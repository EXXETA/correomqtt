package org.correomqtt.core.utils;

import java.io.File;

public class DirectoryUtils {

    private static final String MAC_APP_FOLDER_NAME = "CorreoMqtt";
    private static final String WIN_APP_FOLDER_NAME = MAC_APP_FOLDER_NAME;
    private static final String LIN_APP_FOLDER_NAME = ".correomqtt";
    private static final String USER_DIR = System.getProperty("user.dir");
    private static final String USER_HOME = System.getProperty("user.home");

    private static final String OPERATING_SYSTEM = System.getProperty("os.name").toLowerCase();
    private static String targetDirectoryPathCache;

    public static boolean isWindows() {
        return OPERATING_SYSTEM.startsWith("windows");
    }

    public static boolean isMacOS() {
        return OPERATING_SYSTEM.contains("mac os");
    }

    public static boolean isLinux() {
        return OPERATING_SYSTEM.contains("linux")
                || OPERATING_SYSTEM.contains("mpe/ix")
                || OPERATING_SYSTEM.contains("freebsd")
                || OPERATING_SYSTEM.contains("irix")
                || OPERATING_SYSTEM.contains("digital unix")
                || OPERATING_SYSTEM.contains("unix");
    }

    private DirectoryUtils(){
        // private constructor
    }

    public static String getTargetDirectoryPath() {
        if (targetDirectoryPathCache != null) {
            return targetDirectoryPathCache;
        }
        if (isWindows()) {
            String appData = System.getenv("APPDATA");
            if (appData == null) {
                throw new IllegalStateException("Unable to find windows user directory.");
            } else {
                targetDirectoryPathCache = appData + File.separator + WIN_APP_FOLDER_NAME;
            }
        } else if (isMacOS()) {
            if (USER_HOME == null) {
                throw new IllegalStateException("Unable to find mac OS user directory.");
            } else {
                targetDirectoryPathCache = USER_HOME + File.separator + "Library" + File.separator + "Application Support" + File.separator + MAC_APP_FOLDER_NAME;
            }
        } else if (isLinux()) {
            if (USER_HOME == null) {
                throw new IllegalStateException("Unable to find linux user directory.");
            } else {
                targetDirectoryPathCache = USER_HOME + File.separator + LIN_APP_FOLDER_NAME;
            }
        } else {
            targetDirectoryPathCache = USER_DIR;
        }
        return targetDirectoryPathCache;
    }

    public static String getLogDirectory() {
        return getTargetDirectoryPath() + File.separator;
    }
}
