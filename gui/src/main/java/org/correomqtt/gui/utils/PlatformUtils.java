package org.correomqtt.gui.utils;

import javafx.application.Platform;

public class PlatformUtils {

    private PlatformUtils(){
        // private constructor
    }

    public static void runLaterIfNotInFxThread(Runnable runnable){
        if(Platform.isFxApplicationThread()){
            runnable.run();
        }else {
            Platform.runLater(runnable);
        }
    }
}
