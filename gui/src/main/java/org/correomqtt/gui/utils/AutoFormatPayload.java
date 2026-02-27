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
import java.util.List;
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

        LOGGER.debug("Auto formatting payload: {}", connectionId);

        Format foundFormat = detectFormat(payload);

        if (listener != null) {
            codeArea.textProperty().removeListener(listener);
        }

        try {
            applyFormatting(foundFormat, codeArea, options);
            removeLegacyFormatErrorTooltip(codeArea);
        } catch (Exception e) {
            LOGGER.error("Formatter failed. ", e);
        }

        if (listener != null) {
            codeArea.textProperty().addListener(listener);
        }
        return foundFormat;
    }

    private Format detectFormat(String payload) {
        List<Format> availableFormats = new ArrayList<>(pluginManager.getExtensions(DetailViewFormatHook.class));

        Format validFormat = findFormat(payload, availableFormats, Format::isValid);
        if (validFormat != null) {
            return validFormat;
        }

        Format candidateFormat = findFormat(payload, availableFormats, Format::isCandidate);
        if (candidateFormat != null) {
            return candidateFormat;
        }

        Plain plain = new Plain();
        plain.setText(payload);
        return plain;
    }

    private Format findFormat(String payload, List<Format> formats, FormatPredicate predicate) {
        for (Format format : formats) {
            if (format == null) {
                continue;
            }
            try {
                format.setText(payload);
                if (predicate.test(format)) {
                    return format;
                }
            } catch (Exception e) {
                LOGGER.error("Formatting check failed. ", e);
            }
        }
        return null;
    }

    private void applyFormatting(Format format, CodeArea codeArea, FormatOptions options) {
        if (options.highlightOnly()) {
            if (options.applyHighlighting()) {
                applySyntaxHighlighting(format, codeArea, options.showFormatErrors());
            } else {
                clearSyntaxHighlighting(codeArea);
            }
        } else {
            applyFullFormatting(format, codeArea, options);
        }
    }

    private void applyFullFormatting(Format format, CodeArea codeArea, FormatOptions options) {
        String prettyString = format.getPrettyString();
        if (!Objects.equals(codeArea.getText(), prettyString)) {
            int caretPosition = codeArea.getCaretPosition();
            codeArea.replaceText(prettyString);
            codeArea.moveTo(Math.min(caretPosition, prettyString.length()));
        }
        applySyntaxHighlighting(format, codeArea, options.showFormatErrors());
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

    @FunctionalInterface
    private interface FormatPredicate {
        boolean test(Format format);
    }
}
