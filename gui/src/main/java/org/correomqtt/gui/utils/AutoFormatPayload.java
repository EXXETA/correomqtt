package org.correomqtt.gui.utils;

import javafx.beans.value.ChangeListener;
import javafx.scene.control.Tooltip;
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

        Format foundFormat = null;
        ArrayList<Format> availableFormats = new ArrayList<>(pluginManager.getExtensions(DetailViewFormatHook.class));

        // 1) Prefer valid (fully parseable) formats from plugins
        for (Format format : availableFormats) {
            if (format == null) {
                continue;
            }
            try {
                format.setText(payload);
                if (format.isValid()) {
                    foundFormat = format;
                    break;
                }
            } catch (Exception e) {
                LOGGER.error("Formatting check failed. ", e);
            }
        }

        // 2) If none are valid, pick a candidate format (e.g. JSON while typing with errors)
        if (foundFormat == null) {
            for (Format format : availableFormats) {
                if (format == null) {
                    continue;
                }
                try {
                    format.setText(payload);
                    if (format.isCandidate()) {
                        foundFormat = format;
                        break;
                    }
                } catch (Exception e) {
                    LOGGER.error("Formatting check failed. ", e);
                }
            }
        }

        // 3) Fallback to plain text
        if (foundFormat == null) {
            foundFormat = new Plain();
            foundFormat.setText(payload);
        }


        // ChangeListener<String> listener is needed to disable it when the text of the PublishCodeArea changes. It is reenabled after the manipulation.
        if (listener != null) {
            codeArea.textProperty().removeListener(listener);
        }

        try {
            String prettyString = foundFormat.getPrettyString();
            if (!Objects.equals(codeArea.getText(), prettyString)) {
                int caretPosition = codeArea.getCaretPosition();
                codeArea.replaceText(prettyString);
                codeArea.moveTo(Math.min(caretPosition, prettyString.length()));
            }
            codeArea.setStyleSpans(0, foundFormat.getFxSpans());

            final String tooltipKey = "autoFormatPayload.tooltip";
            Tooltip existingTooltip = (Tooltip) codeArea.getProperties().get(tooltipKey);
            var error = foundFormat.getError();
            if (error.isPresent()) {
                var formatError = error.get();
                String errorText = (formatError.line() > 0 && formatError.column() > 0)
                        ? ("Line " + formatError.line() + ", Column " + formatError.column() + ": " + formatError.message())
                        : formatError.message();
                if (existingTooltip == null) {
                    existingTooltip = new Tooltip();
                    codeArea.getProperties().put(tooltipKey, existingTooltip);
                    Tooltip.install(codeArea, existingTooltip);
                }
                existingTooltip.setText(errorText);
            } else if (existingTooltip != null) {
                Tooltip.uninstall(codeArea, existingTooltip);
                codeArea.getProperties().remove(tooltipKey);
            }
        } catch (Exception e) {
            LOGGER.error("Formatter failed. ", e);
        }

        if (listener != null) {
            codeArea.textProperty().addListener(listener);
        }
        return foundFormat;

    }
}
