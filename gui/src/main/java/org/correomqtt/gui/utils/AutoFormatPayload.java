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
import org.fxmisc.richtext.model.StyleSpan;
import org.fxmisc.richtext.model.StyleSpans;
import org.fxmisc.richtext.model.StyleSpansBuilder;

import java.util.ArrayList;
import java.util.Collection;
import java.util.Collections;
import java.util.Objects;

@DefaultBean
public class AutoFormatPayload {

    private static final Logger LOGGER = LoggerFactory.getLogger(AutoFormatPayload.class);
    private static final String LEGACY_TOOLTIP_KEY = "autoFormatPayload.tooltip";
    private static final String ERROR_STYLE_CLASS = "errorJSON";
    private final PluginManager pluginManager;

    @Inject
    public AutoFormatPayload(PluginManager pluginManager) {
        this.pluginManager = pluginManager;
    }

    public Format autoFormatPayload(final String payload, boolean doFormatting, String connectionId, CodeArea codeArea) {
        return autoFormatPayload(payload, doFormatting, connectionId, codeArea, null, FormatOptions.DEFAULTS);
    }

    public Format autoFormatPayload(final String payload,
                                    boolean doFormatting,
                                    String connectionId,
                                    CodeArea codeArea,
                                    ChangeListener<String> listener,
                                    FormatOptions options) {

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
            if (options.highlightOnly()) {
                // Only apply syntax highlighting, don't replace text
                if (options.applyHighlighting()) {
                    applySyntaxHighlighting(foundFormat, codeArea, options.showFormatErrors());
                } else {
                    clearSyntaxHighlighting(codeArea);
                }
            } else {
                // Full formatting: replace text and apply highlighting
                String prettyString = foundFormat.getPrettyString();
                if (!Objects.equals(codeArea.getText(), prettyString)) {
                    int caretPosition = codeArea.getCaretPosition();
                    codeArea.replaceText(prettyString);
                    codeArea.moveTo(Math.min(caretPosition, prettyString.length()));
                }
                applySyntaxHighlighting(foundFormat, codeArea, options.showFormatErrors());
            }

            removeLegacyFormatErrorTooltip(codeArea);
        } catch (Exception e) {
            LOGGER.error("Formatter failed. ", e);
        }

        if (listener != null) {
            codeArea.textProperty().addListener(listener);
        }
        return foundFormat;

    }

    private void applySyntaxHighlighting(Format foundFormat, CodeArea codeArea, boolean showFormatErrors) {
        String currentText = codeArea.getText();
        foundFormat.setText(currentText);
        StyleSpans<Collection<String>> spans = foundFormat.getFxSpans();
        StyleSpans<Collection<String>> effectiveSpans = showFormatErrors ? spans : removeErrorStyles(spans);
        if (effectiveSpans.length() == currentText.length()) {
            codeArea.setStyleSpans(0, effectiveSpans);
        }
    }

    private StyleSpans<Collection<String>> removeErrorStyles(StyleSpans<Collection<String>> spans) {
        StyleSpansBuilder<Collection<String>> spansBuilder = new StyleSpansBuilder<>();
        for (StyleSpan<Collection<String>> styleSpan : spans) {
            Collection<String> styleClasses = styleSpan.getStyle();
            ArrayList<String> filteredStyleClasses = new ArrayList<>();
            for (String styleClass : styleClasses) {
                if (!Objects.equals(styleClass, ERROR_STYLE_CLASS)) {
                    filteredStyleClasses.add(styleClass);
                }
            }
            Collection<String> spanStyles = filteredStyleClasses.isEmpty()
                    ? Collections.emptyList()
                    : Collections.unmodifiableList(filteredStyleClasses);
            spansBuilder.add(spanStyles, styleSpan.getLength());
        }
        return spansBuilder.create();
    }

    private void clearSyntaxHighlighting(CodeArea codeArea) {
        String currentText = codeArea.getText();
        StyleSpansBuilder<Collection<String>> spansBuilder = new StyleSpansBuilder<>();
        spansBuilder.add(Collections.emptyList(), currentText.length());
        codeArea.setStyleSpans(0, spansBuilder.create());
    }

    private void removeLegacyFormatErrorTooltip(CodeArea codeArea) {
        Tooltip existingTooltip = (Tooltip) codeArea.getProperties().get(LEGACY_TOOLTIP_KEY);
        if (existingTooltip != null) {
            Tooltip.uninstall(codeArea, existingTooltip);
            codeArea.getProperties().remove(LEGACY_TOOLTIP_KEY);
        }
    }
}
