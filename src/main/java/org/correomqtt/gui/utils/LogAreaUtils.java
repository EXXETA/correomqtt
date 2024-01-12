package org.correomqtt.gui.utils;

import org.fxmisc.richtext.CodeArea;

public class LogAreaUtils {

    private LogAreaUtils() {
        // private constructor
    }

    public static void appendColorful(CodeArea logTextArea, String msg) {

        String[] matches = msg.split("\u001B");
        String cssClass;
        for (String match : matches) {
            String str;
            if (match.startsWith("[36m")) {
                cssClass = "cyan";
                str = match.substring(4);
            } else if (match.startsWith("[34m")) {
                cssClass = "blue";
                str = match.substring(4);
            } else if (match.startsWith("[31m")) {
                cssClass = "orange";
                str = match.substring(4);
            } else if (match.startsWith("[33m")) {
                cssClass = "yellow";
                str = match.substring(4);
            } else if (match.startsWith("[35m")) {
                cssClass = "magenta";
                str = match.substring(4);
            } else if (match.startsWith("[1;31m")) {
                cssClass = "red";
                str = match.substring(6);
            } else if (match.startsWith("[0;39m")) {
                cssClass = "default";
                str = match.substring(6);
            } else {
                cssClass = "default";
                str = match;
            }
            logTextArea.append(str, cssClass);
        }
    }
}
