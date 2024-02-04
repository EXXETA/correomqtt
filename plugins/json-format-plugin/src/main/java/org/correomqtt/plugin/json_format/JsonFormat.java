package org.correomqtt.plugin.json_format;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.MapperFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.gui.plugin.spi.DetailViewFormatHook;
import org.fxmisc.richtext.model.StyleSpans;
import org.fxmisc.richtext.model.StyleSpansBuilder;
import org.pf4j.Extension;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.util.Collection;
import java.util.Collections;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@Extension
public class JsonFormat implements DetailViewFormatHook {

    private static final Logger LOGGER = LoggerFactory.getLogger(JsonFormat.class);

    private static final Pattern JSON_PATTERN = Pattern.compile(
            "(?<OBJECT>(?<KEYOBJECT>\"(\\\\.|[^\"])*\")\\h*([:])\\h*([{]))" +
                    "|(?<ARRAY>(?<KEYARRAY>\"(\\\\.|[^\"])*\")\\h*([:])\\h*([\\[]))" +
                    "|(?<NUMBER>(?<KEYNUMBER>\"(\\\\.|[^\"])*\")\\h*([:])\\h*(?<VALUENUMBER>[0-9]+))" +
                    "|(?<NULL>(?<KEYNULL>\"(\\\\.|[^\"])*\")\\h*([:])\\h*(?<VALUENULL>null))" +
                    "|(?<STRING>(?<KEYSTRING>\"(\\\\.|[^\"])*\")\\h*([:])\\h*(?<VALUESTRING>\"(\\\\.|[^\"])*\"))" +
                    "|(?<BOOLEAN>(?<KEYBOOLEAN>\"(\\\\.|[^\"])*\")\\h*([:])\\h*(?<VALUEBOOLEAN>true|false))" +
                    "|(?<SINGLESTRING>\"(\\\\.|[^\"])*\")" +
                    "|(?<SINGLENUMBER>[0-9]+)" +
                    "|(?<SINGLENULL>null)" +
                    "|(?<SINGLEBOOLEAN>true|false)"
    );

    private static final String OBJECT_GROUP = "OBJECT";
    private static final String ARRAY_GROUP = "ARRAY";
    private static final String NUMBER_GROUP = "NUMBER";
    private static final String NULL_GROUP = "NULL";
    private static final String STRING_GROUP = "STRING";
    private static final String BOOLEAN_GROUP = "BOOLEAN";
    private static final String KEYNULL_GROUP = "KEYNULL";
    private static final String VALUENULL_GROUP = "VALUENULL";
    private static final String KEYBOOLEAN_GROUP = "KEYBOOLEAN";
    private static final String VALUEBOOLEAN_GROUP = "VALUEBOOLEAN";
    private static final String KEYSTRING_GROUP = "KEYSTRING";
    private static final String VALUESTRING_GROUP = "VALUESTRING";
    private static final String KEYNUMBER_GROUP = "KEYNUMBER";
    private static final String VALUENUMBER_GROUP = "VALUENUMBER";
    private static final String KEYOBJECT_GROUP = "KEYOBJECT";
    private static final String KEYARRAY_GROUP = "KEYARRAY";
    private static final String SINGLESTRING_GROUP = "SINGLESTRING";
    private static final String SINGLENUMBER_GROUP = "SINGLENUMBER";
    private static final String SINGLENULL_GROUP = "SINGLENULL";
    private static final String SINGLEBOOLEAN_GROUP = "SINGLEBOOLEAN";

    private static final String KEY_CLASS = "keyJSON";
    private static final String STRING_CLASS = "valueJSON";
    private static final String NUMBER_CLASS = "numberJSON";
    private static final String BOOLEAN_CLASS = "booleanJSON";
    private static final String NULL_CLASS = "nullJSON";

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

    private Object createJsonObject() {
        try {
            ObjectMapper objectMapper = new ObjectMapper();
            objectMapper.configure(MapperFeature.CAN_OVERRIDE_ACCESS_MODIFIERS, false);
            jsonObject = objectMapper.readValue(text, Object.class);
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
            ObjectMapper objectMapper = new ObjectMapper();
            objectMapper.configure(MapperFeature.CAN_OVERRIDE_ACCESS_MODIFIERS, false);
            return objectMapper.writerWithDefaultPrettyPrinter().writeValueAsString(getParsedJsonObject());
        } catch (JsonProcessingException e) {
            LOGGER.trace("Could not write pretty JSON. ", e);
            return text;
        }
    }

    @Override
    public StyleSpans<Collection<String>> getFxSpans() {
        String prettyString = getPrettyString();
        StyleSpansBuilder<Collection<String>> spansBuilder = new StyleSpansBuilder<>();

        int lastKwEnd = 0;
        Matcher matcher = JSON_PATTERN.matcher(prettyString);
        while (matcher.find()) {
            spansBuilder.add(Collections.emptyList(), matcher.start() - lastKwEnd);
            if (matcher.group(OBJECT_GROUP) != null) {
                spansBuilder.add(Collections.emptyList(), matcher.start(KEYOBJECT_GROUP) - matcher.start());
                spansBuilder.add(Collections.singleton(KEY_CLASS), matcher.end(KEYOBJECT_GROUP) - matcher.start(KEYOBJECT_GROUP));
                spansBuilder.add(Collections.emptyList(), matcher.end() - matcher.end(KEYOBJECT_GROUP));
            } else if (matcher.group(ARRAY_GROUP) != null) {
                spansBuilder.add(Collections.emptyList(), matcher.start(KEYARRAY_GROUP) - matcher.start());
                spansBuilder.add(Collections.singleton(KEY_CLASS), matcher.end(KEYARRAY_GROUP) - matcher.start(KEYARRAY_GROUP));
                spansBuilder.add(Collections.emptyList(), matcher.end() - matcher.end(KEYARRAY_GROUP));
            } else if (matcher.group(NUMBER_GROUP) != null) {
                spansBuilder.add(Collections.emptyList(), matcher.start(KEYNUMBER_GROUP) - matcher.start());
                spansBuilder.add(Collections.singleton(KEY_CLASS), matcher.end(KEYNUMBER_GROUP) - matcher.start(KEYNUMBER_GROUP));
                spansBuilder.add(Collections.emptyList(), matcher.start(VALUENUMBER_GROUP) - matcher.end(KEYNUMBER_GROUP));
                spansBuilder.add(Collections.singleton(NUMBER_CLASS), matcher.end(VALUENUMBER_GROUP) - matcher.start(VALUENUMBER_GROUP));
                spansBuilder.add(Collections.emptyList(), matcher.end() - matcher.end(VALUENUMBER_GROUP));
            } else if (matcher.group(BOOLEAN_GROUP) != null) {
                spansBuilder.add(Collections.emptyList(), matcher.start(KEYBOOLEAN_GROUP) - matcher.start());
                spansBuilder.add(Collections.singleton(KEY_CLASS), matcher.end(KEYBOOLEAN_GROUP) - matcher.start(KEYBOOLEAN_GROUP));
                spansBuilder.add(Collections.emptyList(), matcher.start(VALUEBOOLEAN_GROUP) - matcher.end(KEYBOOLEAN_GROUP));
                spansBuilder.add(Collections.singleton(BOOLEAN_CLASS), matcher.end(VALUEBOOLEAN_GROUP) - matcher.start(VALUEBOOLEAN_GROUP));
                spansBuilder.add(Collections.emptyList(), matcher.end() - matcher.end(VALUEBOOLEAN_GROUP));
            } else if (matcher.group(NULL_GROUP) != null) {
                spansBuilder.add(Collections.emptyList(), matcher.start(KEYNULL_GROUP) - matcher.start());
                spansBuilder.add(Collections.singleton(KEY_CLASS), matcher.end(KEYNULL_GROUP) - matcher.start(KEYNULL_GROUP));
                spansBuilder.add(Collections.emptyList(), matcher.start(VALUENULL_GROUP) - matcher.end(KEYNULL_GROUP));
                spansBuilder.add(Collections.singleton(NULL_CLASS), matcher.end(VALUENULL_GROUP) - matcher.start(VALUENULL_GROUP));
                spansBuilder.add(Collections.emptyList(), matcher.end() - matcher.end(VALUENULL_GROUP));
            } else if (matcher.group(STRING_GROUP) != null) {
                spansBuilder.add(Collections.emptyList(), matcher.start(KEYSTRING_GROUP) - matcher.start());
                spansBuilder.add(Collections.singleton(KEY_CLASS), matcher.end(KEYSTRING_GROUP) - matcher.start(KEYSTRING_GROUP));
                spansBuilder.add(Collections.emptyList(), matcher.start(VALUESTRING_GROUP) - matcher.end(KEYSTRING_GROUP));
                spansBuilder.add(Collections.singleton(STRING_CLASS), matcher.end(VALUESTRING_GROUP) - matcher.start(VALUESTRING_GROUP));
                spansBuilder.add(Collections.emptyList(), matcher.end() - matcher.end(VALUESTRING_GROUP));
            } else if (matcher.group(SINGLESTRING_GROUP) != null) {
                spansBuilder.add(Collections.singleton(STRING_CLASS), matcher.end() - matcher.start());
            } else if (matcher.group(SINGLENUMBER_GROUP) != null) {
                spansBuilder.add(Collections.singleton(NUMBER_CLASS), matcher.end() - matcher.start());
            } else if (matcher.group(SINGLEBOOLEAN_GROUP) != null) {
                spansBuilder.add(Collections.singleton(BOOLEAN_CLASS), matcher.end() - matcher.start());
            } else if (matcher.group(SINGLENULL_GROUP) != null) {
                spansBuilder.add(Collections.singleton(NULL_CLASS), matcher.end() - matcher.start());
            }
            lastKwEnd = matcher.end();
        }

        return spansBuilder.create();
    }
}
