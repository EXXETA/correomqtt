package org.correomqtt.plugin.json_format;

import com.fasterxml.jackson.core.JsonFactory;
import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.JsonToken;
import com.fasterxml.jackson.core.PrettyPrinter;
import com.fasterxml.jackson.core.util.DefaultIndenter;
import com.fasterxml.jackson.core.util.DefaultPrettyPrinter;
import com.fasterxml.jackson.databind.MapperFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.util.ArrayList;
import java.util.List;
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

    private static final String KEY_CLASS = "keyJSON";
    private static final String STRING_CLASS = "valueJSON";
    private static final String NUMBER_CLASS = "numberJSON";
    private static final String BOOLEAN_CLASS = "booleanJSON";
    private static final String NULL_CLASS = "nullJSON";
    private static ObjectMapper OBJECT_MAPPER;
    private static final PrettyPrinter PRETTY_PRINTER = new DefaultPrettyPrinter().withObjectIndenter(new DefaultIndenter().withLinefeed("\n"));

    private String text;
    private Object jsonObject;

    @Override
    public void setText(String text) {
        this.text = text;
        this.jsonObject = createJsonObject();
    }

    @Override
    public boolean isValid() {
        return getParsedJsonObject() != null;
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
        } catch (IOException e) {
            LOGGER.trace("JSON could not be parsed. ", e);
            return null;
        }

        return jsonObject;
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

        for (JsonMatch match : getMatches(prettyString)) {
            if (match.getStart() > lastPos) {
                int length = match.getStart() - lastPos;
                spansBuilder.add(Collections.emptyList(), length);
            }

            // add current token with style, then remember pos for next iteration
            spansBuilder.add(Collections.singleton(match.getType()), match.getEnd() - match.getStart());
            lastPos = match.getEnd();
        }

        // prevent exception if empty string
        if (lastPos == 0) {
            spansBuilder.add(Collections.emptyList(), prettyString.length());
        }

        return spansBuilder.create();
    }
}
