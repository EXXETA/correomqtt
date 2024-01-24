package org.correomqtt.gui.utils;

import javafx.application.Platform;
import lombok.NonNull;
import org.apache.commons.lang3.ArrayUtils;
import org.fxmisc.richtext.CodeArea;

import java.io.OutputStream;
import java.util.Arrays;

public class CodeAreaOutputStream extends OutputStream {
    private final CodeArea codeArea;

    public CodeAreaOutputStream(CodeArea codeArea) {
        this.codeArea = codeArea;
    }

    @Override
    public void write(int i) {
        Platform.runLater(() -> codeArea.appendText(String.valueOf((char) i)));
    }

    @Override
    public void write(byte @NonNull [] b, int off, int len) {
        Platform.runLater(() -> codeArea.appendText(Arrays.toString(ArrayUtils.subarray(b, off, off + len))));
    }
}