package org.correomqtt.plugin.json_format;

import com.fasterxml.jackson.core.JsonFactory;
import com.fasterxml.jackson.core.JsonLocation;
import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.JsonToken;
import com.fasterxml.jackson.core.PrettyPrinter;
import com.fasterxml.jackson.core.util.DefaultIndenter;
import com.fasterxml.jackson.core.util.DefaultPrettyPrinter;
import com.fasterxml.jackson.databind.MapperFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Deque;
import java.util.HashSet;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.Set;
import org.correomqtt.gui.formats.Format;
import org.correomqtt.gui.plugin.spi.DetailViewFormatHook;
import org.fxmisc.richtext.model.StyleSpans;
import org.fxmisc.richtext.model.StyleSpansBuilder;
import org.pf4j.Extension;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.util.Collection;
import java.util.Collections;

@Extension
public class JsonFormat implements DetailViewFormatHook {

    private static final Logger LOGGER = LoggerFactory.getLogger(JsonFormat.class);
    private static final int MAX_AUTO_FIX_PASSES = 8;

    private static final String KEY_CLASS = "keyJSON";
    private static final String STRING_CLASS = "valueJSON";
    private static final String NUMBER_CLASS = "numberJSON";
    private static final String BOOLEAN_CLASS = "booleanJSON";
    private static final String NULL_CLASS = "nullJSON";
    private static final String ERROR_CLASS = "errorJSON";

    private static ObjectMapper OBJECT_MAPPER;
    private static final PrettyPrinter PRETTY_PRINTER = new DefaultPrettyPrinter().withObjectIndenter(new DefaultIndenter().withLinefeed("\n"));

    private String text;
    private Object jsonObject;
    private Format.FormatError formatError;

    @Override
    public void setText(String text) {
        this.text = text;
        this.formatError = null;
        this.jsonObject = createJsonObject();
    }

    @Override
    public boolean isValid() {
        return getParsedJsonObject() != null;
    }

    @Override
    public boolean isCandidate() {
        return looksLikeJson();
    }

    /**
     * Check if the text looks like it could be JSON (starts with { or [)
     * This allows partial formatting even with errors
     */
    public boolean looksLikeJson() {
        if (text == null || text.isBlank()) {
            return false;
        }
        String trimmed = text.trim();
        return trimmed.startsWith("{") || trimmed.startsWith("[");
    }

    private Object getParsedJsonObject() {

        if (jsonObject == null) {
            jsonObject = createJsonObject();
        }

        return jsonObject;
    }

    private static ObjectMapper getObjectMapper() {

        if (OBJECT_MAPPER == null) {

            OBJECT_MAPPER = new ObjectMapper();
            OBJECT_MAPPER.configure(MapperFeature.CAN_OVERRIDE_ACCESS_MODIFIERS, false);
        }

        return OBJECT_MAPPER;
    }

    private Object createJsonObject() {

        try {
            jsonObject = getObjectMapper().readValue(text, Object.class);
            formatError = null;
        } catch (JsonProcessingException e) {
            LOGGER.trace("JSON could not be parsed. ", e);
            JsonLocation location = e.getLocation();
            if (location != null) {
                formatError = new Format.FormatError(
                    (int) location.getLineNr(),
                    (int) location.getColumnNr(),
                    (int) location.getCharOffset(),
                    extractErrorMessage(e.getOriginalMessage())
                );
            } else {
                formatError = new Format.FormatError(-1, -1, -1, extractErrorMessage(e.getMessage()));
            }
            return null;
        } catch (IOException e) {
            LOGGER.trace("JSON could not be parsed. ", e);
            formatError = new Format.FormatError(-1, -1, -1, e.getMessage());
            return null;
        }

        return jsonObject;
    }

    /**
     * Extracts a user-friendly error message from Jackson's error output
     */
    private String extractErrorMessage(String fullMessage) {
        if (fullMessage == null) {
            return "Unknown error";
        }
        // Jackson messages often have format: "message\n at [Source: ...]"
        int atIndex = fullMessage.indexOf("\n at [Source");
        if (atIndex > 0) {
            return fullMessage.substring(0, atIndex).trim();
        }
        // Also try without newline
        atIndex = fullMessage.indexOf(" at [Source");
        if (atIndex > 0) {
            return fullMessage.substring(0, atIndex).trim();
        }
        return fullMessage;
    }

    @Override
    public String getPrettyString() {

        if (getParsedJsonObject() == null) {
            return text;
        }

        try {
            return getObjectMapper().writer(PRETTY_PRINTER).writeValueAsString(getParsedJsonObject());
        } catch (JsonProcessingException e) {
            LOGGER.trace("Could not write pretty JSON. ", e);
            return text;
        }
    }

    @Override
    public String getMinifiedString() {
        if (getParsedJsonObject() == null) {
            return text;
        }

        try {
            return getObjectMapper().writeValueAsString(getParsedJsonObject());
        } catch (JsonProcessingException e) {
            LOGGER.trace("Could not minify JSON. ", e);
            return text;
        }
    }

    @Override
    public Optional<Format.FormatError> getError() {
        // Ensure we've tried to parse
        getParsedJsonObject();
        return Optional.ofNullable(formatError);
    }

    @Override
    public String getAutoFixedString() {
        if (text == null) {
            return null;
        }

        String candidate = normalizeCommonArtifacts(text);
        boolean normalizedChanged = !Objects.equals(candidate, text);
        Set<String> seenCandidates = new HashSet<>();
        seenCandidates.add(candidate);
        int executedPasses = 0;

        for (int pass = 1; pass <= MAX_AUTO_FIX_PASSES; pass++) {
            executedPasses = pass;
            String nextCandidate = applySingleFixPass(candidate);
            if (nextCandidate == null) {
                return text;
            }

            if (LOGGER.isDebugEnabled()) {
                LOGGER.debug(
                        "JSON auto-fix pass {}: normalizedChanged={}, passChanged={}",
                        pass,
                        normalizedChanged,
                        !Objects.equals(nextCandidate, candidate)
                );
            }

            candidate = nextCandidate;

            try {
                Object parsed = getObjectMapper().readValue(candidate, Object.class);
                if (LOGGER.isDebugEnabled()) {
                    LOGGER.debug("JSON auto-fix pass {}: parseSuccess=true, outputChanged={}", pass, !Objects.equals(candidate, text));
                }
                return getObjectMapper().writer(PRETTY_PRINTER).writeValueAsString(parsed);
            } catch (Exception e) {
                LOGGER.trace("Could not auto-fix JSON in pass {}. ", pass, e);
            }

            if (pass == MAX_AUTO_FIX_PASSES) {
                break;
            }

            if (!seenCandidates.add(candidate)) {
                if (LOGGER.isDebugEnabled()) {
                    LOGGER.debug("JSON auto-fix stopped: detected repeated candidate after pass {}.", pass);
                }
                break;
            }
        }

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug(
                    "JSON auto-fix pipeline: parseSuccess=false after {} pass(es), returning best-effort candidate (changed={}).",
                    executedPasses,
                    !Objects.equals(candidate, text)
            );
        }
        return !Objects.equals(candidate, text) ? candidate : text;
    }

    private String applySingleFixPass(String input) {
        String fixed = fixUnquotedStrings(input);
        if (fixed == null) {
            return null;
        }
        fixed = removeTrailingCommas(fixed);
        fixed = closeOpenContainers(fixed);
        return fixed;
    }

    private enum Container {
        OBJECT,
        ARRAY
    }

    private enum Expect {
        KEY_OR_END,
        VALUE_OR_END,
        COLON,
        VALUE,
        COMMA_OR_END
    }

    private String fixUnquotedStrings(String input) {
        StringBuilder out = new StringBuilder(input.length() + 16);

        Deque<Container> stack = new ArrayDeque<>();
        Expect expect = Expect.VALUE;

        int i = 0;
        while (i < input.length()) {
            char c = input.charAt(i);

            // whitespace
            if (Character.isWhitespace(c)) {
                out.append(c);
                i++;
                continue;
            }

            if (expect == Expect.COMMA_OR_END && c != ',' && c != '}' && c != ']') {
                insertCommaBeforeTrailingWhitespace(out);
                if (!stack.isEmpty() && stack.peek() == Container.OBJECT) {
                    expect = Expect.KEY_OR_END;
                } else {
                    expect = Expect.VALUE_OR_END;
                }
                continue;
            }

            if ((c == '{' || c == '[') && expect == Expect.KEY_OR_END) {
                if (LOGGER.isDebugEnabled()) {
                    LOGGER.debug("JSON auto-fix: ambiguous object key before index {}, aborting token fix.", i);
                }
                return null;
            }

            // structure
            if (c == '{') {
                out.append(c);
                stack.push(Container.OBJECT);
                expect = Expect.KEY_OR_END;
                i++;
                continue;
            }
            if (c == '[') {
                out.append(c);
                stack.push(Container.ARRAY);
                expect = Expect.VALUE_OR_END;
                i++;
                continue;
            }
            if (c == '}' || c == ']') {
                if (stack.isEmpty()) {
                    if (LOGGER.isDebugEnabled()) {
                        LOGGER.debug("JSON auto-fix: dropped unmatched closer '{}' at index {}.", c, i);
                    }
                    i++;
                    continue;
                }

                Container expectedContainer = stack.peek();
                char expectedCloser = expectedContainer == Container.OBJECT ? '}' : ']';
                if (c != expectedCloser) {
                    out.append(expectedCloser);
                    stack.pop();
                    expect = Expect.COMMA_OR_END;
                    if (LOGGER.isDebugEnabled()) {
                        LOGGER.debug(
                                "JSON auto-fix: replaced mismatched closer '{}' with '{}' at index {}.",
                                c,
                                expectedCloser,
                                i
                        );
                    }
                    continue;
                }

                out.append(c);
                stack.pop();
                expect = Expect.COMMA_OR_END;
                i++;
                continue;
            }
            if (c == ':') {
                out.append(c);
                expect = Expect.VALUE;
                i++;
                continue;
            }
            if (expect == Expect.COLON && c == '=') {
                out.append(':');
                expect = Expect.VALUE;
                i++;
                continue;
            }
            if (expect == Expect.COLON && c != ':' && c != ',' && c != '}' && c != ']') {
                out.append(':');
                expect = Expect.VALUE;
                if (LOGGER.isDebugEnabled()) {
                    LOGGER.debug("JSON auto-fix: inserted missing colon before index {}.", i);
                }
                continue;
            }
            if (c == ',') {
                out.append(c);
                if (!stack.isEmpty() && stack.peek() == Container.OBJECT) {
                    expect = Expect.KEY_OR_END;
                } else {
                    expect = Expect.VALUE_OR_END;
                }
                i++;
                continue;
            }

            // quoted string
            if (c == '"') {
                int start = i;
                i++;
                boolean escaped = false;
                boolean isClosed = false;
                boolean usedFallbackClosingQuote = false;
                while (i < input.length()) {
                    char currentCharacter = input.charAt(i);
                    if (escaped) {
                        escaped = false;
                    } else if (currentCharacter == '\\') {
                        escaped = true;
                    } else if (currentCharacter == '"') {
                        i++;
                        isClosed = true;
                        break;
                    } else if (currentCharacter == '\'' && isLikelyWrongClosingSingleQuote(input, i)) {
                        out.append(input, start, i).append('"');
                        i++;
                        isClosed = true;
                        usedFallbackClosingQuote = true;
                        if (LOGGER.isDebugEnabled()) {
                            LOGGER.debug("JSON auto-fix: repaired mismatched single quote at index {}.", i - 1);
                        }
                        break;
                    } else if ((currentCharacter == '\n' || currentCharacter == '\r') && isLikelyImplicitStringEndBeforeLineBreak(input, i)) {
                        out.append(input, start, i).append('"');
                        isClosed = true;
                        usedFallbackClosingQuote = true;
                        if (LOGGER.isDebugEnabled()) {
                            LOGGER.debug("JSON auto-fix: closed unterminated string before line break at index {}.", i);
                        }
                        break;
                    }
                    i++;
                }

                if (!isClosed) {
                    if (i >= input.length() && !escaped) {
                        out.append(input, start, i).append('"');
                        isClosed = true;
                        usedFallbackClosingQuote = true;
                        if (LOGGER.isDebugEnabled()) {
                            LOGGER.debug("JSON auto-fix: closed unterminated string at end of input.");
                        }
                    }
                }

                if (!isClosed) {
                    if (LOGGER.isDebugEnabled()) {
                        LOGGER.debug("JSON auto-fix: found unterminated string without unambiguous end, aborting token fix.");
                    }
                    return input;
                }

                if (!usedFallbackClosingQuote) {
                    out.append(input, start, i);
                }
                expect = (expect == Expect.KEY_OR_END) ? Expect.COLON : Expect.COMMA_OR_END;
                continue;
            }

            // single-quoted string -> convert to double quotes
            if (c == '\'') {
                i++;
                StringBuilder s = new StringBuilder();
                boolean escaped = false;
                while (i < input.length()) {
                    char cc = input.charAt(i);
                    if (escaped) {
                        s.append(cc);
                        escaped = false;
                    } else if (cc == '\\') {
                        escaped = true;
                    } else if (cc == '\'') {
                        i++;
                        break;
                    } else {
                        s.append(cc);
                    }
                    i++;
                }
                out.append('"').append(s.toString().replace("\\", "\\\\").replace("\"", "\\\"")).append('"');
                expect = (expect == Expect.KEY_OR_END) ? Expect.COLON : Expect.COMMA_OR_END;
                continue;
            }

            // bare token
            int start = i;
            while (i < input.length()) {
                char cc = input.charAt(i);
                if (Character.isWhitespace(cc) || cc == '{' || cc == '}' || cc == '[' || cc == ']' || cc == ':' || cc == ',' || cc == '"' || cc == '\'') {
                    break;
                }
                i++;
            }
            String token = input.substring(start, i);
            if (token.isEmpty()) {
                out.append(c);
                i++;
                continue;
            }

            boolean tokenIsLiteral = token.equals("true") || token.equals("false") || token.equals("null");
            boolean tokenIsNumber = isNumberToken(token);

            boolean quoteAsKey = (expect == Expect.KEY_OR_END);
            boolean quoteAsValue = (expect == Expect.VALUE || expect == Expect.VALUE_OR_END);

            if (quoteAsKey) {
                out.append('"').append(escapeJsonString(token)).append('"');
                expect = Expect.COLON;
            } else if (quoteAsValue) {
                if (tokenIsLiteral || tokenIsNumber) {
                    out.append(token);
                } else {
                    out.append('"').append(escapeJsonString(token)).append('"');
                }
                expect = Expect.COMMA_OR_END;
            } else {
                out.append(token);
            }
        }

        return out.toString();
    }

    private void insertCommaBeforeTrailingWhitespace(StringBuilder output) {
        int insertionIndex = output.length();
        while (insertionIndex > 0 && Character.isWhitespace(output.charAt(insertionIndex - 1))) {
            insertionIndex--;
        }
        output.insert(insertionIndex, ',');
    }

    private boolean isNumberToken(String token) {
        try {
            Double.parseDouble(token);
            return true;
        } catch (Exception e) {
            return false;
        }
    }

    private String escapeJsonString(String s) {
        return s.replace("\\", "\\\\").replace("\"", "\\\"");
    }

    private boolean isLikelyWrongClosingSingleQuote(String input, int quoteIndex) {
        int nextIndex = quoteIndex + 1;
        while (nextIndex < input.length() && Character.isWhitespace(input.charAt(nextIndex))) {
            nextIndex++;
        }

        if (nextIndex >= input.length()) {
            return true;
        }

        char nextCharacter = input.charAt(nextIndex);
        if (nextCharacter == ',' || nextCharacter == '}' || nextCharacter == ']') {
            return true;
        }

        if (nextCharacter == '"') {
            return looksLikeNextJsonKey(input, nextIndex);
        }

        return false;
    }

    private boolean isLikelyImplicitStringEndBeforeLineBreak(String input, int lineBreakIndex) {
        int nextIndex = lineBreakIndex;
        while (nextIndex < input.length() && Character.isWhitespace(input.charAt(nextIndex))) {
            nextIndex++;
        }

        if (nextIndex >= input.length()) {
            return true;
        }

        char nextCharacter = input.charAt(nextIndex);
        if (nextCharacter == ',' || nextCharacter == '}' || nextCharacter == ']') {
            return true;
        }

        if (nextCharacter == '"') {
            return looksLikeNextJsonKey(input, nextIndex);
        }

        return false;
    }

    private boolean looksLikeNextJsonKey(String input, int keyQuoteIndex) {
        int index = keyQuoteIndex + 1;
        boolean escaped = false;
        while (index < input.length()) {
            char currentCharacter = input.charAt(index);
            if (escaped) {
                escaped = false;
            } else if (currentCharacter == '\\') {
                escaped = true;
            } else if (currentCharacter == '"') {
                index++;
                while (index < input.length() && Character.isWhitespace(input.charAt(index))) {
                    index++;
                }
                if (index >= input.length()) {
                    return false;
                }
                char separator = input.charAt(index);
                return separator == ':' || separator == '=';
            }
            index++;
        }
        return false;
    }

    private String normalizeCommonArtifacts(String input) {
        String normalized = input;
        if (!normalized.isEmpty() && normalized.charAt(0) == '\uFEFF') {
            normalized = normalized.substring(1);
        }

        normalized = normalized
                .replace('\u201C', '"')
                .replace('\u201D', '"')
                .replace('\u2018', '\'')
                .replace('\u2019', '\'');

        StringBuilder cleaned = new StringBuilder(normalized.length());
        for (int index = 0; index < normalized.length(); index++) {
            char currentCharacter = normalized.charAt(index);
            boolean allowedControlCharacter = currentCharacter == '\n'
                    || currentCharacter == '\r'
                    || currentCharacter == '\t';
            if (Character.isISOControl(currentCharacter) && !allowedControlCharacter) {
                continue;
            }
            cleaned.append(currentCharacter);
        }
        return cleaned.toString();
    }

    private String removeTrailingCommas(String input) {
        StringBuilder output = new StringBuilder(input.length());
        int index = 0;
        boolean inString = false;
        boolean escaped = false;

        while (index < input.length()) {
            char currentCharacter = input.charAt(index);
            if (inString) {
                output.append(currentCharacter);
                if (escaped) {
                    escaped = false;
                } else if (currentCharacter == '\\') {
                    escaped = true;
                } else if (currentCharacter == '"') {
                    inString = false;
                }
                index++;
                continue;
            }

            if (currentCharacter == '"') {
                inString = true;
                output.append(currentCharacter);
                index++;
                continue;
            }

            if (currentCharacter == ',') {
                int lookAheadIndex = index + 1;
                while (lookAheadIndex < input.length() && Character.isWhitespace(input.charAt(lookAheadIndex))) {
                    lookAheadIndex++;
                }
                if (lookAheadIndex < input.length()) {
                    char lookAheadCharacter = input.charAt(lookAheadIndex);
                    if (lookAheadCharacter == '}' || lookAheadCharacter == ']') {
                        index++;
                        continue;
                    }
                }
            }

            output.append(currentCharacter);
            index++;
        }

        return output.toString();
    }

    private String closeOpenContainers(String input) {
        Deque<Character> closers = new ArrayDeque<>();
        boolean inString = false;
        boolean escaped = false;

        for (int index = 0; index < input.length(); index++) {
            char currentCharacter = input.charAt(index);
            if (inString) {
                if (escaped) {
                    escaped = false;
                } else if (currentCharacter == '\\') {
                    escaped = true;
                } else if (currentCharacter == '"') {
                    inString = false;
                }
                continue;
            }

            if (currentCharacter == '"') {
                inString = true;
                continue;
            }

            if (currentCharacter == '{') {
                closers.push('}');
                continue;
            }
            if (currentCharacter == '[') {
                closers.push(']');
                continue;
            }

            if ((currentCharacter == '}' || currentCharacter == ']') && !closers.isEmpty() && closers.peek() == currentCharacter) {
                closers.pop();
            }
        }

        if (closers.isEmpty()) {
            return input;
        }

        StringBuilder output = new StringBuilder(input.length() + closers.size());
        output.append(input);
        while (!closers.isEmpty()) {
            output.append(closers.pop());
        }
        return output.toString();
    }

    public static String mapJsonToStyle(JsonToken jsonToken) {

        if (jsonToken == null) {
            return "";
        }

        return switch (jsonToken) {
            case FIELD_NAME -> KEY_CLASS;
            case VALUE_STRING -> STRING_CLASS;
            case VALUE_NUMBER_FLOAT, VALUE_NUMBER_INT -> NUMBER_CLASS;
            case VALUE_NULL -> NULL_CLASS;
            case VALUE_TRUE, VALUE_FALSE -> BOOLEAN_CLASS;
            default -> "";
        };
    }

    private List<JsonMatch> getMatches(String json) {

        List<JsonMatch> matches = new ArrayList<>();

        try (JsonParser parser = new JsonFactory().createParser(json)) {
            while (!parser.isClosed()) {
                var token = parser.nextToken();
                int start = (int) parser.getTokenLocation().getCharOffset();
                int end = start + parser.getTextLength();

                // parser does not include " by default
                if (token == JsonToken.VALUE_STRING || token == JsonToken.FIELD_NAME) {
                    end += 2;
                }

                String styleClass = mapJsonToStyle(token);

                if (!styleClass.isEmpty()) {
                    JsonMatch match = new JsonMatch(styleClass, start, end);
                    matches.add(match);
                }
            }
        } catch (IOException e) {
            // if not valid json, just ignore
        }

        return matches;
    }

    @Override
    public StyleSpans<Collection<String>> getFxSpans() {

        StyleSpansBuilder<Collection<String>> spansBuilder = new StyleSpansBuilder<>();
        var prettyString = getPrettyString();
        int lastPos = 0;

        List<JsonMatch> matches = getMatches(prettyString);

        // If there's an error and we have its position, add error highlighting
        int errorCharOffset = -1;
        if (formatError != null && formatError.charOffset() >= 0) {
            errorCharOffset = formatError.charOffset();
        }

        for (JsonMatch match : matches) {
            if (match.getStart() > lastPos) {
                int length = match.getStart() - lastPos;
                // Check if error is in this gap
                if (errorCharOffset >= lastPos && errorCharOffset < match.getStart()) {
                    // Split around error position
                    int beforeError = errorCharOffset - lastPos;
                    if (beforeError > 0) {
                        spansBuilder.add(Collections.emptyList(), beforeError);
                    }
                    // Mark error region (rest of gap)
                    int errorLength = match.getStart() - errorCharOffset;
                    if (errorLength > 0) {
                        spansBuilder.add(Collections.singleton(ERROR_CLASS), errorLength);
                    }
                } else {
                    spansBuilder.add(Collections.emptyList(), length);
                }
            }

            // Check if error is within this token
            if (errorCharOffset >= match.getStart() && errorCharOffset < match.getEnd()) {
                spansBuilder.add(Arrays.asList(match.getType(), ERROR_CLASS), match.getEnd() - match.getStart());
            } else {
                spansBuilder.add(Collections.singleton(match.getType()), match.getEnd() - match.getStart());
            }
            lastPos = match.getEnd();
        }

        // Handle remaining text after last match
        if (lastPos < prettyString.length()) {
            int remaining = prettyString.length() - lastPos;
            if (errorCharOffset >= lastPos) {
                // Error is in remaining text
                int beforeError = Math.max(0, errorCharOffset - lastPos);
                if (beforeError > 0) {
                    spansBuilder.add(Collections.emptyList(), beforeError);
                }
                spansBuilder.add(Collections.singleton(ERROR_CLASS), remaining - beforeError);
            } else {
                spansBuilder.add(Collections.emptyList(), remaining);
            }
        }

        // prevent exception if empty string
        if (lastPos == 0 && prettyString.isEmpty()) {
            spansBuilder.add(Collections.emptyList(), 0);
        } else if (lastPos == 0) {
            // No matches found but we have text - might be invalid JSON
            if (errorCharOffset >= 0 && errorCharOffset < prettyString.length()) {
                if (errorCharOffset > 0) {
                    spansBuilder.add(Collections.emptyList(), errorCharOffset);
                }
                spansBuilder.add(Collections.singleton(ERROR_CLASS), prettyString.length() - errorCharOffset);
            } else {
                spansBuilder.add(Collections.emptyList(), prettyString.length());
            }
        }

        return spansBuilder.create();
    }
}
