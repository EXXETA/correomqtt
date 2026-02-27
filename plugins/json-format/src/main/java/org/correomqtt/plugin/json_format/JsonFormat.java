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
            formatError = buildFormatError(e);
            return null;
        }
        return jsonObject;
    }

    private Format.FormatError buildFormatError(JsonProcessingException exception) {
        JsonLocation location = exception.getLocation();
        if (location != null) {
            return new Format.FormatError(
                    location.getLineNr(),
                    location.getColumnNr(),
                    (int) location.getCharOffset(),
                    extractErrorMessage(exception.getOriginalMessage())
            );
        }
        return new Format.FormatError(-1, -1, -1, extractErrorMessage(exception.getMessage()));
    }

    private String extractErrorMessage(String fullMessage) {
        if (fullMessage == null) {
            return "Unknown error";
        }
        int atIndex = fullMessage.indexOf("\n at [Source");
        if (atIndex > 0) {
            return fullMessage.substring(0, atIndex).trim();
        }
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
        getParsedJsonObject();
        return Optional.ofNullable(formatError);
    }

    @Override
    public String getAutoFixedString() {
        if (text == null) {
            return null;
        }

        String candidate = normalizeCommonArtifacts(text);
        Set<String> seenCandidates = new HashSet<>();
        seenCandidates.add(candidate);

        for (int pass = 1; pass <= MAX_AUTO_FIX_PASSES; pass++) {
            String nextCandidate = applySingleFixPass(candidate);
            if (nextCandidate == null) {
                return text;
            }

            LOGGER.debug("JSON auto-fix pass {}: passChanged={}", pass, !Objects.equals(nextCandidate, candidate));
            candidate = nextCandidate;

            String prettified = tryParseAndPrettify(candidate);
            if (prettified != null) {
                return prettified;
            }

            if (pass == MAX_AUTO_FIX_PASSES || !seenCandidates.add(candidate)) {
                break;
            }
        }

        return !Objects.equals(candidate, text) ? candidate : text;
    }

    private String tryParseAndPrettify(String candidate) {
        try {
            Object parsed = getObjectMapper().readValue(candidate, Object.class);
            return getObjectMapper().writer(PRETTY_PRINTER).writeValueAsString(parsed);
        } catch (Exception e) {
            LOGGER.trace("Could not parse JSON candidate. ", e);
            return null;
        }
    }

    private String applySingleFixPass(String input) {
        String fixed = new JsonTokenFixer(input).fix();
        if (fixed == null) {
            return null;
        }
        fixed = removeTrailingCommas(fixed);
        fixed = closeOpenContainers(fixed);
        return fixed;
    }

    // ── Token fixer state machine ──────────────────────────────────────

    private enum Container { OBJECT, ARRAY }

    private enum Expect { KEY_OR_END, VALUE_OR_END, COLON, VALUE, COMMA_OR_END }

    private static final class JsonTokenFixer {

        private final String input;
        private final StringBuilder out;
        private final Deque<Container> stack;
        private int position;
        private Expect expect;

        private JsonTokenFixer(String input) {
            this.input = input;
            this.out = new StringBuilder(input.length() + 16);
            this.stack = new ArrayDeque<>();
            this.position = 0;
            this.expect = Expect.VALUE;
        }

        private String fix() {
            while (position < input.length()) {
                char currentCharacter = input.charAt(position);

                if (Character.isWhitespace(currentCharacter)) {
                    out.append(currentCharacter);
                    position++;
                } else if (!processToken(currentCharacter)) {
                    return null;
                }
            }
            return out.toString();
        }

        private boolean processToken(char currentCharacter) {
            if (handleMissingComma(currentCharacter)) {
                return true;
            }
            if (currentCharacter == '{' || currentCharacter == '[') {
                return handleContainerOpen(currentCharacter);
            }
            if (currentCharacter == '}' || currentCharacter == ']') {
                return handleContainerClose(currentCharacter);
            }
            if (currentCharacter == ':') {
                return handleColon();
            }
            if (expect == Expect.COLON) {
                return handleExpectedColon(currentCharacter);
            }
            if (currentCharacter == ',') {
                return handleComma();
            }
            if (currentCharacter == '"') {
                return handleDoubleQuotedString();
            }
            if (currentCharacter == '\'') {
                return handleSingleQuotedString();
            }
            return handleBareToken();
        }

        private boolean handleMissingComma(char currentCharacter) {
            if (expect != Expect.COMMA_OR_END || currentCharacter == ',' || currentCharacter == '}' || currentCharacter == ']') {
                return false;
            }
            insertCommaBeforeTrailingWhitespace(out);
            expect = nextExpectAfterComma();
            return true;
        }

        private boolean handleContainerOpen(char currentCharacter) {
            if (expect == Expect.KEY_OR_END) {
                LOGGER.debug("JSON auto-fix: ambiguous object key before index {}, aborting token fix.", position);
                return false;
            }
            out.append(currentCharacter);
            if (currentCharacter == '{') {
                stack.push(Container.OBJECT);
                expect = Expect.KEY_OR_END;
            } else {
                stack.push(Container.ARRAY);
                expect = Expect.VALUE_OR_END;
            }
            position++;
            return true;
        }

        private boolean handleContainerClose(char currentCharacter) {
            if (stack.isEmpty()) {
                LOGGER.debug("JSON auto-fix: dropped unmatched closer '{}' at index {}.", currentCharacter, position);
                position++;
                return true;
            }

            char expectedCloser = stack.peek() == Container.OBJECT ? '}' : ']';
            if (currentCharacter != expectedCloser) {
                out.append(expectedCloser);
                stack.pop();
                expect = Expect.COMMA_OR_END;
                LOGGER.debug("JSON auto-fix: replaced mismatched closer '{}' with '{}' at index {}.", currentCharacter, expectedCloser, position);
                return true;
            }

            out.append(currentCharacter);
            stack.pop();
            expect = Expect.COMMA_OR_END;
            position++;
            return true;
        }

        private boolean handleColon() {
            out.append(':');
            expect = Expect.VALUE;
            position++;
            return true;
        }

        private boolean handleExpectedColon(char currentCharacter) {
            if (currentCharacter == '=') {
                out.append(':');
                expect = Expect.VALUE;
                position++;
                return true;
            }
            if (currentCharacter != ',' && currentCharacter != '}' && currentCharacter != ']') {
                out.append(':');
                expect = Expect.VALUE;
                LOGGER.debug("JSON auto-fix: inserted missing colon before index {}.", position);
                return true;
            }
            return processToken(currentCharacter);
        }

        private boolean handleComma() {
            out.append(',');
            expect = nextExpectAfterComma();
            position++;
            return true;
        }

        private boolean handleDoubleQuotedString() {
            int start = position;
            position++;
            boolean closed = scanDoubleQuotedString(start);

            if (!closed) {
                LOGGER.debug("JSON auto-fix: found unterminated string without unambiguous end, aborting token fix.");
                out.setLength(0);
                out.append(input);
                return true;
            }

            expect = (expect == Expect.KEY_OR_END) ? Expect.COLON : Expect.COMMA_OR_END;
            return true;
        }

        private boolean scanDoubleQuotedString(int start) {
            boolean escaped = false;

            while (position < input.length()) {
                char currentCharacter = input.charAt(position);
                if (escaped) {
                    escaped = false;
                    position++;
                } else if (currentCharacter == '\\') {
                    escaped = true;
                    position++;
                } else if (currentCharacter == '"') {
                    out.append(input, start, ++position);
                    return true;
                } else if (currentCharacter == '\'' && isLikelyWrongClosingSingleQuote(input, position)) {
                    out.append(input, start, position).append('"');
                    position++;
                    return true;
                } else if (isLineBreak(currentCharacter) && isLikelyImplicitStringEndBeforeLineBreak(input, position)) {
                    out.append(input, start, position).append('"');
                    return true;
                } else {
                    position++;
                }
            }

            if (!escaped) {
                out.append(input, start, position).append('"');
                return true;
            }

            return false;
        }

        private boolean handleSingleQuotedString() {
            position++;
            StringBuilder content = new StringBuilder();
            boolean escaped = false;
            while (position < input.length()) {
                char currentCharacter = input.charAt(position);
                if (escaped) {
                    content.append(currentCharacter);
                    escaped = false;
                } else if (currentCharacter == '\\') {
                    escaped = true;
                } else if (currentCharacter == '\'') {
                    position++;
                    break;
                } else {
                    content.append(currentCharacter);
                }
                position++;
            }
            out.append('"').append(escapeJsonString(content.toString())).append('"');
            expect = (expect == Expect.KEY_OR_END) ? Expect.COLON : Expect.COMMA_OR_END;
            return true;
        }

        private boolean handleBareToken() {
            int start = position;
            while (position < input.length() && !isTokenDelimiter(input.charAt(position))) {
                position++;
            }
            String token = input.substring(start, position);
            if (token.isEmpty()) {
                out.append(input.charAt(start));
                position = start + 1;
                return true;
            }

            appendBareToken(token);
            return true;
        }

        private void appendBareToken(String token) {
            boolean isLiteral = token.equals("true") || token.equals("false") || token.equals("null");
            boolean isNumber = isNumberToken(token);

            if (expect == Expect.KEY_OR_END) {
                out.append('"').append(escapeJsonString(token)).append('"');
                expect = Expect.COLON;
            } else if (expect == Expect.VALUE || expect == Expect.VALUE_OR_END) {
                if (isLiteral || isNumber) {
                    out.append(token);
                } else {
                    out.append('"').append(escapeJsonString(token)).append('"');
                }
                expect = Expect.COMMA_OR_END;
            } else {
                out.append(token);
            }
        }

        private Expect nextExpectAfterComma() {
            if (!stack.isEmpty() && stack.peek() == Container.OBJECT) {
                return Expect.KEY_OR_END;
            }
            return Expect.VALUE_OR_END;
        }

        private static void insertCommaBeforeTrailingWhitespace(StringBuilder output) {
            int insertionIndex = output.length();
            while (insertionIndex > 0 && Character.isWhitespace(output.charAt(insertionIndex - 1))) {
                insertionIndex--;
            }
            output.insert(insertionIndex, ',');
        }

        private static boolean isNumberToken(String token) {
            try {
                Double.parseDouble(token);
                return true;
            } catch (Exception e) {
                return false;
            }
        }

        private static String escapeJsonString(String s) {
            return s.replace("\\", "\\\\").replace("\"", "\\\"");
        }

        private static boolean isLineBreak(char character) {
            return character == '\n' || character == '\r';
        }

        private static boolean isTokenDelimiter(char character) {
            return Character.isWhitespace(character)
                    || character == '{' || character == '}'
                    || character == '[' || character == ']'
                    || character == ':' || character == ','
                    || character == '"' || character == '\'';
        }

        private static boolean isLikelyWrongClosingSingleQuote(String input, int quoteIndex) {
            char nextNonWs = nextNonWhitespace(input, quoteIndex + 1);
            return isStructuralEndChar(nextNonWs)
                    || (nextNonWs == '"' && looksLikeNextJsonKey(input, skipWhitespace(input, quoteIndex + 1)));
        }

        private static boolean isLikelyImplicitStringEndBeforeLineBreak(String input, int lineBreakIndex) {
            char nextNonWs = nextNonWhitespace(input, lineBreakIndex);
            return isStructuralEndChar(nextNonWs)
                    || (nextNonWs == '"' && looksLikeNextJsonKey(input, skipWhitespace(input, lineBreakIndex)));
        }

        private static boolean isStructuralEndChar(char character) {
            return character == ',' || character == '}' || character == ']' || character == '\0';
        }

        private static int skipWhitespace(String input, int startIndex) {
            int index = startIndex;
            while (index < input.length() && Character.isWhitespace(input.charAt(index))) {
                index++;
            }
            return index;
        }

        private static boolean looksLikeNextJsonKey(String input, int keyQuoteIndex) {
            int index = keyQuoteIndex + 1;
            boolean escaped = false;
            while (index < input.length()) {
                char currentCharacter = input.charAt(index);
                if (escaped) {
                    escaped = false;
                } else if (currentCharacter == '\\') {
                    escaped = true;
                } else if (currentCharacter == '"') {
                    char separator = nextNonWhitespace(input, index + 1);
                    return separator == ':' || separator == '=';
                }
                index++;
            }
            return false;
        }
    }

    // ── Shared helpers ─────────────────────────────────────────────────

    private static char nextNonWhitespace(String input, int startIndex) {
        int index = startIndex;
        while (index < input.length() && Character.isWhitespace(input.charAt(index))) {
            index++;
        }
        return index < input.length() ? input.charAt(index) : '\0';
    }

    // ── Trailing comma removal ─────────────────────────────────────────

    private String removeTrailingCommas(String input) {
        StringBuilder output = new StringBuilder(input.length());
        StringScanner scanner = new StringScanner(input);

        while (scanner.hasNext()) {
            if (scanner.isInString()) {
                output.append(scanner.consumeAndAdvanceInString());
            } else if (scanner.current() == '"') {
                scanner.enterString();
                output.append(scanner.consumeAndAdvance());
            } else if (scanner.current() == ',' && isTrailingComma(input, scanner.index())) {
                scanner.advance();
            } else {
                output.append(scanner.consumeAndAdvance());
            }
        }

        return output.toString();
    }

    private boolean isTrailingComma(String input, int commaIndex) {
        char nextChar = nextNonWhitespace(input, commaIndex + 1);
        return nextChar == '}' || nextChar == ']';
    }

    // ── Close open containers ──────────────────────────────────────────

    private String closeOpenContainers(String input) {
        Deque<Character> closers = new ArrayDeque<>();
        StringScanner scanner = new StringScanner(input);

        while (scanner.hasNext()) {
            if (scanner.isInString()) {
                scanner.advanceInString();
            } else if (scanner.current() == '"') {
                scanner.enterString();
                scanner.advance();
            } else {
                trackContainerChar(scanner.current(), closers);
                scanner.advance();
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

    private void trackContainerChar(char character, Deque<Character> closers) {
        if (character == '{') {
            closers.push('}');
        } else if (character == '[') {
            closers.push(']');
        } else if ((character == '}' || character == ']') && !closers.isEmpty() && closers.peek() == character) {
            closers.pop();
        }
    }

    // ── String-aware scanner ───────────────────────────────────────────

    private static final class StringScanner {
        private final String input;
        private int index;
        private boolean inString;
        private boolean escaped;

        private StringScanner(String input) {
            this.input = input;
            this.index = 0;
            this.inString = false;
            this.escaped = false;
        }

        private boolean hasNext() {
            return index < input.length();
        }

        private char current() {
            return input.charAt(index);
        }

        private int index() {
            return index;
        }

        private boolean isInString() {
            return inString;
        }

        private void enterString() {
            inString = true;
        }

        private void advance() {
            index++;
        }

        private char consumeAndAdvance() {
            return input.charAt(index++);
        }

        private char consumeAndAdvanceInString() {
            char currentCharacter = input.charAt(index);
            if (escaped) {
                escaped = false;
            } else if (currentCharacter == '\\') {
                escaped = true;
            } else if (currentCharacter == '"') {
                inString = false;
            }
            index++;
            return currentCharacter;
        }

        private void advanceInString() {
            char currentCharacter = input.charAt(index);
            if (escaped) {
                escaped = false;
            } else if (currentCharacter == '\\') {
                escaped = true;
            } else if (currentCharacter == '"') {
                inString = false;
            }
            index++;
        }
    }

    // ── Normalization ──────────────────────────────────────────────────

    private String normalizeCommonArtifacts(String input) {
        String normalized = stripBom(input);
        normalized = replaceSmartQuotes(normalized);
        return stripControlCharacters(normalized);
    }

    private String stripBom(String input) {
        if (!input.isEmpty() && input.charAt(0) == '\uFEFF') {
            return input.substring(1);
        }
        return input;
    }

    private String replaceSmartQuotes(String input) {
        return input
                .replace('\u201C', '"')
                .replace('\u201D', '"')
                .replace('\u2018', '\'')
                .replace('\u2019', '\'');
    }

    private String stripControlCharacters(String input) {
        StringBuilder cleaned = new StringBuilder(input.length());
        for (int index = 0; index < input.length(); index++) {
            char currentCharacter = input.charAt(index);
            if (!Character.isISOControl(currentCharacter) || isAllowedControlCharacter(currentCharacter)) {
                cleaned.append(currentCharacter);
            }
        }
        return cleaned.toString();
    }

    private boolean isAllowedControlCharacter(char character) {
        return character == '\n' || character == '\r' || character == '\t';
    }

    // ── Style mapping ──────────────────────────────────────────────────

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
                JsonToken token = parser.nextToken();
                int start = (int) parser.getTokenLocation().getCharOffset();
                int end = start + parser.getTextLength();
                if (token == JsonToken.VALUE_STRING || token == JsonToken.FIELD_NAME) {
                    end += 2;
                }
                String styleClass = mapJsonToStyle(token);
                if (!styleClass.isEmpty()) {
                    matches.add(new JsonMatch(styleClass, start, end));
                }
            }
        } catch (IOException e) {
            // Intentionally empty: invalid JSON produces no matches
        }
        return matches;
    }

    // ── Syntax highlighting spans ──────────────────────────────────────

    @Override
    public StyleSpans<Collection<String>> getFxSpans() {
        StyleSpansBuilder<Collection<String>> spansBuilder = new StyleSpansBuilder<>();
        String prettyString = getPrettyString();
        List<JsonMatch> matches = getMatches(prettyString);
        int errorOffset = resolveErrorOffset();

        int lastPos = buildMatchSpans(spansBuilder, matches, errorOffset);
        buildTrailingSpans(spansBuilder, lastPos, prettyString, errorOffset);

        return spansBuilder.create();
    }

    private int resolveErrorOffset() {
        if (formatError != null && formatError.charOffset() >= 0) {
            return formatError.charOffset();
        }
        return -1;
    }

    private int buildMatchSpans(StyleSpansBuilder<Collection<String>> spansBuilder,
                                List<JsonMatch> matches, int errorOffset) {
        int lastPos = 0;
        for (JsonMatch match : matches) {
            if (match.getStart() > lastPos) {
                addGapSpan(spansBuilder, lastPos, match.getStart(), errorOffset);
            }
            addTokenSpan(spansBuilder, match, errorOffset);
            lastPos = match.getEnd();
        }
        return lastPos;
    }

    private void addGapSpan(StyleSpansBuilder<Collection<String>> spansBuilder,
                            int gapStart, int gapEnd, int errorOffset) {
        if (errorOffset >= gapStart && errorOffset < gapEnd) {
            int beforeError = errorOffset - gapStart;
            if (beforeError > 0) {
                spansBuilder.add(Collections.emptyList(), beforeError);
            }
            spansBuilder.add(Collections.singleton(ERROR_CLASS), gapEnd - errorOffset);
        } else {
            spansBuilder.add(Collections.emptyList(), gapEnd - gapStart);
        }
    }

    private void addTokenSpan(StyleSpansBuilder<Collection<String>> spansBuilder,
                              JsonMatch match, int errorOffset) {
        int length = match.getEnd() - match.getStart();
        if (errorOffset >= match.getStart() && errorOffset < match.getEnd()) {
            spansBuilder.add(Arrays.asList(match.getType(), ERROR_CLASS), length);
        } else {
            spansBuilder.add(Collections.singleton(match.getType()), length);
        }
    }

    private void buildTrailingSpans(StyleSpansBuilder<Collection<String>> spansBuilder,
                                    int lastPos, String prettyString, int errorOffset) {
        if (lastPos < prettyString.length()) {
            addRemainingSpan(spansBuilder, lastPos, prettyString.length(), errorOffset);
        } else if (lastPos == 0) {
            addNoMatchSpan(spansBuilder, prettyString, errorOffset);
        }
    }

    private void addRemainingSpan(StyleSpansBuilder<Collection<String>> spansBuilder,
                                  int lastPos, int textLength, int errorOffset) {
        int remaining = textLength - lastPos;
        if (errorOffset >= lastPos) {
            int beforeError = Math.max(0, errorOffset - lastPos);
            if (beforeError > 0) {
                spansBuilder.add(Collections.emptyList(), beforeError);
            }
            spansBuilder.add(Collections.singleton(ERROR_CLASS), remaining - beforeError);
        } else {
            spansBuilder.add(Collections.emptyList(), remaining);
        }
    }

    private void addNoMatchSpan(StyleSpansBuilder<Collection<String>> spansBuilder,
                                String prettyString, int errorOffset) {
        if (prettyString.isEmpty()) {
            spansBuilder.add(Collections.emptyList(), 0);
        } else if (errorOffset >= 0 && errorOffset < prettyString.length()) {
            if (errorOffset > 0) {
                spansBuilder.add(Collections.emptyList(), errorOffset);
            }
            spansBuilder.add(Collections.singleton(ERROR_CLASS), prettyString.length() - errorOffset);
        } else {
            spansBuilder.add(Collections.emptyList(), prettyString.length());
        }
    }
}
