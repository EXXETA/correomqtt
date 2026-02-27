package org.correomqtt.plugin.json_format;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;

class JsonFormatTest {

    private static final ObjectMapper OBJECT_MAPPER = new ObjectMapper();

    @Test
    void shouldAutoFixMismatchedClosingQuoteInStringValue() throws Exception {
        // Arrange
        String malformedJson = """
                {
                  "part" : {
                    "persId" : "6DB2C0B7A68B32FEE81CB12DD1822818'
                  },
                  "reason" : "NOT_FOUND",
                  "fbsType" : "FBS5"
                }""";
        JsonFormat jsonFormat = new JsonFormat();
        jsonFormat.setText(malformedJson);

        // Act
        String fixedJson = jsonFormat.getAutoFixedString();
        JsonNode parsedJson = OBJECT_MAPPER.readTree(fixedJson);

        // Assert
        assertEquals("6DB2C0B7A68B32FEE81CB12DD1822818", parsedJson.path("part").path("persId").asText());
        assertEquals("NOT_FOUND", parsedJson.path("reason").asText());
        assertEquals("FBS5", parsedJson.path("fbsType").asText());
    }

    @Test
    void shouldAutoFixMissingCommaBetweenFields() throws Exception {
        // Arrange
        String malformedJson = "{ \"alpha\": 1 \"beta\": 2 }";
        JsonFormat jsonFormat = new JsonFormat();
        jsonFormat.setText(malformedJson);

        // Act
        String fixedJson = jsonFormat.getAutoFixedString();
        JsonNode parsedJson = OBJECT_MAPPER.readTree(fixedJson);

        // Assert
        assertEquals(1, parsedJson.path("alpha").asInt());
        assertEquals(2, parsedJson.path("beta").asInt());
    }

    @Test
    void shouldAutoFixTrailingCommas() throws Exception {
        // Arrange
        String malformedJson = "{ \"values\": [1, 2,], \"flag\": true, }";
        JsonFormat jsonFormat = new JsonFormat();
        jsonFormat.setText(malformedJson);

        // Act
        String fixedJson = jsonFormat.getAutoFixedString();
        JsonNode parsedJson = OBJECT_MAPPER.readTree(fixedJson);

        // Assert
        assertEquals(2, parsedJson.path("values").size());
        assertEquals(2, parsedJson.path("values").get(1).asInt());
        assertEquals(true, parsedJson.path("flag").asBoolean());
    }

    @Test
    void shouldAutoCloseMissingContainers() throws Exception {
        // Arrange
        String malformedJson = "{ \"outer\": { \"items\": [1, 2, 3]";
        JsonFormat jsonFormat = new JsonFormat();
        jsonFormat.setText(malformedJson);

        // Act
        String fixedJson = jsonFormat.getAutoFixedString();
        JsonNode parsedJson = OBJECT_MAPPER.readTree(fixedJson);

        // Assert
        assertEquals(3, parsedJson.path("outer").path("items").size());
        assertEquals(3, parsedJson.path("outer").path("items").get(2).asInt());
    }

    @Test
    void shouldAutoFixEqualsAsSeparator() throws Exception {
        // Arrange
        String malformedJson = "{ \"type\" = \"FBS5\", \"count\" = 10 }";
        JsonFormat jsonFormat = new JsonFormat();
        jsonFormat.setText(malformedJson);

        // Act
        String fixedJson = jsonFormat.getAutoFixedString();
        JsonNode parsedJson = OBJECT_MAPPER.readTree(fixedJson);

        // Assert
        assertEquals("FBS5", parsedJson.path("type").asText());
        assertEquals(10, parsedJson.path("count").asInt());
    }

    @Test
    void shouldNormalizeBomAndSmartQuotes() throws Exception {
        // Arrange
        String malformedJson = "\uFEFF{\u201Cstatus\u201D: \u201COK\u201D, \u2018key\u2019: \u2018value\u2019}";
        JsonFormat jsonFormat = new JsonFormat();
        jsonFormat.setText(malformedJson);

        // Act
        String fixedJson = jsonFormat.getAutoFixedString();
        JsonNode parsedJson = OBJECT_MAPPER.readTree(fixedJson);

        // Assert
        assertEquals("OK", parsedJson.path("status").asText());
        assertEquals("value", parsedJson.path("key").asText());
    }

    @Test
    void shouldFixMismatchedSingleQuoteBeforeNextKey() throws Exception {
        // Arrange
        String malformedJson = """
                {
                  "part" : {
                    "persId" : "6DB2C0B7A68B32FEE81CB12DD1822818"
                  },
                  "reason" : "NOT_FOUND'
                  "fbsType" : "FBS5"
                }""";
        JsonFormat jsonFormat = new JsonFormat();
        jsonFormat.setText(malformedJson);

        // Act
        String fixedJson = jsonFormat.getAutoFixedString();
        JsonNode parsedJson = OBJECT_MAPPER.readTree(fixedJson);

        // Assert
        assertEquals("NOT_FOUND", parsedJson.path("reason").asText());
        assertEquals("FBS5", parsedJson.path("fbsType").asText());
    }

    @Test
    void shouldFixUnterminatedStringBeforeClosingBrace() throws Exception {
        // Arrange
        String malformedJson = """
                {
                  "a": "OK",
                  "b": "unterminated
                }""";
        JsonFormat jsonFormat = new JsonFormat();
        jsonFormat.setText(malformedJson);

        // Act
        String fixedJson = jsonFormat.getAutoFixedString();
        JsonNode parsedJson = OBJECT_MAPPER.readTree(fixedJson);

        // Assert
        assertEquals("OK", parsedJson.path("a").asText());
        assertEquals("unterminated", parsedJson.path("b").asText());
    }

    @Test
    void shouldInsertMissingCommaBeforeLineBreakInsteadOfLeadingNextLine() throws Exception {
        // Arrange
        String malformedJson = """
                {
                  "a": "OK"
                  "b": "value",
                  "c" "missingColon"
                }""";
        JsonFormat jsonFormat = new JsonFormat();
        jsonFormat.setText(malformedJson);

        // Act
        String fixedJson = jsonFormat.getAutoFixedString();
        JsonNode parsedJson = OBJECT_MAPPER.readTree(fixedJson);

        // Assert
        assertEquals("OK", parsedJson.path("a").asText());
        assertEquals("value", parsedJson.path("b").asText());
        assertEquals("missingColon", parsedJson.path("c").asText());
        assertFalse(fixedJson.contains("\n  ,\"b\": \"value\""));
    }

    @Test
    void shouldAutoFixMissingColonBetweenKeyAndQuotedValue() throws Exception {
        // Arrange
        String malformedJson = "{ \"c\" \"missingColon\" }";
        JsonFormat jsonFormat = new JsonFormat();
        jsonFormat.setText(malformedJson);

        // Act
        String fixedJson = jsonFormat.getAutoFixedString();
        JsonNode parsedJson = OBJECT_MAPPER.readTree(fixedJson);

        // Assert
        assertEquals("missingColon", parsedJson.path("c").asText());
    }

    @Test
    void shouldAutoFixCombinedQuoteCommaAndColonIssuesToValidJson() throws Exception {
        // Arrange
        String malformedJson = """
                {
                  "a": "OK'
                  "b": "value",
                  "c" "missingColon"
                }""";
        JsonFormat jsonFormat = new JsonFormat();
        jsonFormat.setText(malformedJson);

        // Act
        String fixedJson = jsonFormat.getAutoFixedString();
        JsonNode parsedJson = OBJECT_MAPPER.readTree(fixedJson);

        // Assert
        assertEquals("OK", parsedJson.path("a").asText());
        assertEquals("value", parsedJson.path("b").asText());
        assertEquals("missingColon", parsedJson.path("c").asText());
    }

    @Test
    void shouldCloseUnterminatedStringAtEndOfInput() throws Exception {
        // Arrange
        String malformedJson = "{ \"a\": \"text";
        JsonFormat jsonFormat = new JsonFormat();
        jsonFormat.setText(malformedJson);

        // Act
        String fixedJson = jsonFormat.getAutoFixedString();
        JsonNode parsedJson = OBJECT_MAPPER.readTree(fixedJson);

        // Assert
        assertEquals("text", parsedJson.path("a").asText());
    }

    @Test
    void shouldLeaveAmbiguousNestedStructureUnchanged() {
        // Arrange
        String malformedJson = """
                {
                  a: b: {[
                \t
                }
                ]}""";
        JsonFormat jsonFormat = new JsonFormat();
        jsonFormat.setText(malformedJson);

        // Act
        String fixedJson = jsonFormat.getAutoFixedString();

        // Assert
        assertEquals(malformedJson, fixedJson);
    }
}
