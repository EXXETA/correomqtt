package org.correomqtt.gui.formats;

import org.fxmisc.richtext.model.StyleSpans;

import java.util.Collection;
import java.util.Optional;

public interface Format {

    void setText(String text);

    boolean isValid();

    String getPrettyString();

    StyleSpans<Collection<String>> getFxSpans();

    default boolean isFormatable() {
        return true;
    }

    /**
     * Returns true if this format is a good candidate for the currently set text,
     * even if {@link #isValid()} is false (e.g. while typing and the content is temporarily invalid).
     */
    default boolean isCandidate() {
        return isValid();
    }

    /**
     * Returns error information if the format validation failed.
     *
     * @return Optional containing error info, or empty if no error
     */
    default Optional<FormatError> getError() {
        return Optional.empty();
    }

    /**
     * Returns the minified version of the content.
     *
     * @return minified string, or original if minification not supported/failed
     */
    default String getMinifiedString() {
        return getPrettyString();
    }

    /**
     * Returns the escaped version of the content for embedding in strings.
     *
     * @return escaped string
     */
    default String getEscapedString() {
        return getPrettyString()
                .replace("\\", "\\\\")
                .replace("\"", "\\\"")
                .replace("\n", "\\n")
                .replace("\r", "\\r")
                .replace("\t", "\\t");
    }

    /**
     * Attempts to automatically fix common issues (e.g. unquoted strings) and returns a prettified result.
     *
     * @return fixed/prettified string, or original if not supported/failed
     */
    default String getAutoFixedString() {
        return getPrettyString();
    }

    /**
     * Record to hold format error information
     */
    record FormatError(int line, int column, int charOffset, String message) {
    }
}
