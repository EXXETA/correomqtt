package org.correomqtt.business.utils;

import javafx.beans.value.ChangeListener;
import org.correomqtt.gui.formats.Format;
import org.correomqtt.gui.formats.Plain;
import org.correomqtt.plugin.manager.PluginManager;
import org.correomqtt.plugin.spi.DetailViewFormatHook;
import org.fxmisc.richtext.CodeArea;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import java.util.ArrayList;
import java.util.Objects;

public class AutoFormatPayload {

    private static final Logger LOGGER = LoggerFactory.getLogger(CorreoCharsetDecoder.class);

    private AutoFormatPayload() {
        // private Constructor
    }

    public static Format autoFormatPayload(final String payload, boolean doFormatting, String connectionId, CodeArea codeArea) {
        return autoFormatPayload(payload, doFormatting, connectionId, codeArea, null);
    }

    public static Format autoFormatPayload(final String payload, boolean doFormatting, String connectionId, CodeArea codeArea, ChangeListener<String> listener) {

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Auto formatting payload: {}", connectionId);
        }

        Format foundFormat;
        if (doFormatting) {
            // Find the first format that is valid.
            ArrayList<Format> availableFormats = new ArrayList<>(PluginManager.getInstance().getExtensions(DetailViewFormatHook.class));
            availableFormats.add(new Plain());
            foundFormat = availableFormats.stream()
                    .filter(Objects::nonNull)
                    .filter(format -> {
                                format.setText(payload);
                                return format.isValid();
                            }
                    )
                    .findFirst()
                    .orElseThrow(() -> new IllegalArgumentException("Plain format did not match."));
        } else {
            foundFormat = new Plain();
            foundFormat.setText(payload);
        }

        //ChangeListener<String> listener is needed to disable it when the text of the PublishCodeArea changes. It is reenabled after the manipulation.
        if (listener != null) {
            codeArea.textProperty().removeListener(listener);
        }

        codeArea.clear();
        codeArea.replaceText(0, 0, foundFormat.getPrettyString());
        codeArea.setStyleSpans(0, foundFormat.getFxSpans());

        if (listener != null) {
            codeArea.textProperty().addListener(listener);
        }
        return foundFormat;

    }
}
