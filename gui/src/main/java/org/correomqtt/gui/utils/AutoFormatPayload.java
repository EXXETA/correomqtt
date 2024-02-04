package org.correomqtt.gui.utils;

import javafx.beans.value.ChangeListener;
import org.correomqtt.core.plugin.PluginManager;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.gui.formats.Format;
import org.correomqtt.gui.formats.Plain;
import org.correomqtt.gui.plugin.spi.DetailViewFormatHook;
import org.fxmisc.richtext.CodeArea;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import org.correomqtt.di.Inject;
import java.util.ArrayList;
import java.util.Objects;

@DefaultBean
public class AutoFormatPayload {

    private static final Logger LOGGER = LoggerFactory.getLogger(AutoFormatPayload.class);
    private final PluginManager pluginManager;

    @Inject
    public AutoFormatPayload(PluginManager pluginManager) {
        // private Constructor
        this.pluginManager = pluginManager;
    }

    public Format autoFormatPayload(final String payload, boolean doFormatting, String connectionId, CodeArea codeArea) {
        return autoFormatPayload(payload, doFormatting, connectionId, codeArea, null);
    }

    public Format autoFormatPayload(final String payload, boolean doFormatting, String connectionId, CodeArea codeArea, ChangeListener<String> listener) {

        if (!doFormatting) {
            return null;
        }

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Auto formatting payload: {}", connectionId);
        }

        Format foundFormat;
        // Find the first format that is valid.
        ArrayList<Format> availableFormats = new ArrayList<>(pluginManager.getExtensions(DetailViewFormatHook.class));
        availableFormats.add(new Plain());
        foundFormat = availableFormats.stream()
                .filter(Objects::nonNull)
                .filter(format -> {
                            try {
                                format.setText(payload);
                                return format.isValid();
                            } catch (Exception e) {
                                LOGGER.error("Formatting check failed. ", e);
                                return false;
                            }
                        }
                )
                .findFirst()
                .orElseThrow(() -> new IllegalArgumentException("Plain format did not match."));


        //ChangeListener<String> listener is needed to disable it when the text of the PublishCodeArea changes. It is reenabled after the manipulation.
        if (listener != null) {
            codeArea.textProperty().removeListener(listener);
        }

        codeArea.clear();
        try {
            codeArea.replaceText(0, 0, foundFormat.getPrettyString());
            codeArea.setStyleSpans(0, foundFormat.getFxSpans());
        } catch (Exception e) {
            LOGGER.error("Formatter failed. ", e);
            foundFormat = new Plain();
            foundFormat.setText(payload);
            codeArea.replaceText(0, 0, foundFormat.getPrettyString());
            codeArea.setStyleSpans(0, foundFormat.getFxSpans());
        }

        if (listener != null) {
            codeArea.textProperty().addListener(listener);
        }
        return foundFormat;

    }
}
