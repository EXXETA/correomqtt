package org.correomqtt.gui.utils;

import javafx.event.EventHandler;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import org.fxmisc.richtext.CodeArea;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.Map;
import java.util.Set;

/**
 * Utility class for adding auto-complete functionality to CodeArea components.
 * Supports auto-completion of brackets, braces, and quotes for JSON editing.
 */
public class CodeAreaAutoComplete {

    private static final String KEY_TYPED_HANDLER = "codeAreaAutoComplete.keyTypedHandler";
    private static final String KEY_PRESSED_HANDLER = "codeAreaAutoComplete.keyPressedHandler";

    private static final Logger LOGGER = LoggerFactory.getLogger(CodeAreaAutoComplete.class);

    private static final Map<String, String> PAIRS = Map.of(
            "\"", "\"",
            "{", "}",
            "[", "]"
    );

    private static final Set<String> CLOSING_CHARS = Set.of("\"", "}", "]");

    private CodeAreaAutoComplete() {
        // Utility class
    }

    /**
     * Sets up auto-complete functionality for a CodeArea.
     * - Auto-completes opening brackets/braces/quotes with their closing counterpart
     * - Places cursor between the pair
     * - Skips over closing characters if they're already present
     * - Deletes both characters when backspace is pressed on an empty pair
     *
     * @param codeArea The CodeArea to enable auto-complete on
     */
    public static void setupAutoComplete(CodeArea codeArea) {
        disableAutoComplete(codeArea);

        EventHandler<KeyEvent> typedHandler = event -> {
            String typed = event.getCharacter();

            // Auto-Complete for opening characters
            if (PAIRS.containsKey(typed)) {
                handleOpeningChar(codeArea, typed, event);
            }
            // Skip closing characters if already present
            else if (CLOSING_CHARS.contains(typed)) {
                handleClosingChar(codeArea, typed, event);
            }
        };

        // Backspace: Delete both characters if pair is empty
        EventHandler<KeyEvent> pressedHandler = event -> {
            if (event.getCode() == KeyCode.BACK_SPACE) {
                handleBackspace(codeArea, event);
            }
        };

        codeArea.getProperties().put(KEY_TYPED_HANDLER, typedHandler);
        codeArea.getProperties().put(KEY_PRESSED_HANDLER, pressedHandler);

        codeArea.addEventFilter(KeyEvent.KEY_TYPED, typedHandler);
        codeArea.addEventFilter(KeyEvent.KEY_PRESSED, pressedHandler);

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Auto-complete setup completed for CodeArea");
        }
    }

    /**
     * Disables auto-complete functionality for a CodeArea.
     *
     * @param codeArea The CodeArea to disable auto-complete on
     */
    @SuppressWarnings("unchecked")
    public static void disableAutoComplete(CodeArea codeArea) {
        EventHandler<KeyEvent> typedHandler = (EventHandler<KeyEvent>) codeArea.getProperties().remove(KEY_TYPED_HANDLER);
        if (typedHandler != null) {
            codeArea.removeEventFilter(KeyEvent.KEY_TYPED, typedHandler);
        }

        EventHandler<KeyEvent> pressedHandler = (EventHandler<KeyEvent>) codeArea.getProperties().remove(KEY_PRESSED_HANDLER);
        if (pressedHandler != null) {
            codeArea.removeEventFilter(KeyEvent.KEY_PRESSED, pressedHandler);
        }
    }

    private static void handleOpeningChar(CodeArea codeArea, String typed, KeyEvent event) {
        int caretPos = codeArea.getCaretPosition();
        String closing = PAIRS.get(typed);

        // For quotes, check if we're inside a string
        if (typed.equals("\"")) {
            String text = codeArea.getText();
            if (isInsideString(text, caretPos)) {
                return; // Normal behavior inside a string
            }
        }

        // Check if next character is already the closing one (to avoid double-insertion)
        String text = codeArea.getText();
        if (caretPos < text.length()) {
            char nextChar = text.charAt(caretPos);
            // For quotes, if next char is closing quote and we're closing a string, don't auto-complete
            if (typed.equals("\"") && nextChar == '"') {
                return;
            }
        }

        // Consume event and insert pair
        event.consume();
        codeArea.insertText(caretPos, typed + closing);
        codeArea.moveTo(caretPos + 1); // Place cursor between the pair
    }

    private static void handleClosingChar(CodeArea codeArea, String typed, KeyEvent event) {
        int caretPos = codeArea.getCaretPosition();
        String text = codeArea.getText();

        // If the next character is the same closing character, skip over it
        if (caretPos < text.length() &&
                String.valueOf(text.charAt(caretPos)).equals(typed)) {
            event.consume();
            codeArea.moveTo(caretPos + 1);
        }
    }

    private static void handleBackspace(CodeArea codeArea, KeyEvent event) {
        int caretPos = codeArea.getCaretPosition();
        String text = codeArea.getText();

        if (caretPos > 0 && caretPos < text.length()) {
            String before = String.valueOf(text.charAt(caretPos - 1));
            String after = String.valueOf(text.charAt(caretPos));

            // If we're between a pair, delete both
            if (PAIRS.containsKey(before) && PAIRS.get(before).equals(after)) {
                event.consume();
                codeArea.deleteText(caretPos - 1, caretPos + 1);
            }
        }
    }

    /**
     * Checks if the given position is inside a JSON string.
     * Counts unescaped quotes before the position.
     *
     * @param text The full text content
     * @param position The position to check
     * @return true if inside a string, false otherwise
     */
    private static boolean isInsideString(String text, int position) {
        int quoteCount = 0;
        for (int i = 0; i < position && i < text.length(); i++) {
            char c = text.charAt(i);
            if (c == '"' && (i == 0 || text.charAt(i - 1) != '\\')) {
                quoteCount++;
            }
        }
        return quoteCount % 2 == 1;
    }
}
