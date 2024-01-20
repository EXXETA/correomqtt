package org.correomqtt.gui.utils;

import javafx.application.Platform;
import org.fxmisc.richtext.CodeArea;

import java.io.OutputStream;

public class CodeAreaOutputStream extends OutputStream {
    private final CodeArea codeArea;

    public CodeAreaOutputStream(CodeArea codeArea) {
        this.codeArea = codeArea;
    }

    @Override
    public void write(int i) {
        Platform.runLater(() -> codeArea.appendText(String.valueOf((char) i)));
    }

}