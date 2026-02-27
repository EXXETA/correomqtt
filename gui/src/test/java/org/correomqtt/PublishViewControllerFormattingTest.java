package org.correomqtt;

import javafx.beans.value.ChangeListener;
import javafx.scene.Scene;
import javafx.scene.control.ToggleButton;
import javafx.scene.layout.StackPane;
import javafx.stage.Stage;
import org.correomqtt.gui.formats.Format;
import org.correomqtt.gui.formats.Plain;
import org.correomqtt.gui.utils.AutoFormatPayload;
import org.correomqtt.gui.utils.FormatOptions;
import org.correomqtt.gui.views.connections.PublishViewController;
import org.fxmisc.richtext.CodeArea;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.mockito.Mockito;
import org.testfx.framework.junit5.ApplicationTest;

import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.util.Collection;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;

class PublishViewControllerFormattingTest extends ApplicationTest {

    private PublishViewController publishViewController;
    private CodeArea payloadCodeArea;

    @Override
    public void start(Stage stage) {
        StackPane rootPane = new StackPane();
        Scene scene = new Scene(rootPane, 200, 100);
        stage.setScene(scene);
        stage.show();
    }

    @BeforeEach
    void setUp() throws Exception {
        // Arrange
        publishViewController = Mockito.mock(PublishViewController.class, Mockito.CALLS_REAL_METHODS);
        assertNotNull(publishViewController);

        interact(() -> payloadCodeArea = new CodeArea());

        ToggleButton highlightingToggleButton = new ToggleButton();
        ChangeListener<String> payloadChangeListener = (observable, oldValue, newValue) -> {
        };

        setField(publishViewController, "payloadCodeArea", payloadCodeArea);
        setField(publishViewController, "highlightingToggleButton", highlightingToggleButton);
        setField(publishViewController, "payloadCodeAreaChangeListener", payloadChangeListener);
        setField(publishViewController, "autoFormatPayload", new NoOpAutoFormatPayload());
    }

    /* TODO: Move into plugin UI contributions
    @Test
    void shouldApplyAutoFixJsonTextToPayloadCodeArea() throws Exception {
        // Arrange
        String malformedPayload = "{foo:bar}";
        String fixedPayload = "{\n  \"foo\": \"bar\"\n}";
        TestFormat format = new TestFormat(fixedPayload, fixedPayload, fixedPayload);
        setField(publishViewController, "currentFormat", format);
        setField(publishViewController, "autoFormatPayload", new FixedFormatAutoFormatPayload(format));
        interact(() -> payloadCodeArea.replaceText(malformedPayload));

        // Act
        invokeControllerMethod(publishViewController, "onFixJson");

        // Assert
        assertEquals(fixedPayload, payloadCodeArea.getText());
    }

    @Test
    void shouldEscapePayloadTextInCodeArea() throws Exception {
        // Arrange
        String originalPayload = "He said \"hi\"\n";
        String escapedPayload = "He said \\\"hi\\\"\\n";
        interact(() -> payloadCodeArea.replaceText(originalPayload));

        // Act
        invokeControllerMethod(publishViewController, "onEscape");

        // Assert
        assertEquals(escapedPayload, payloadCodeArea.getText());
    }

    @Test
    void shouldUnescapePayloadTextInCodeArea() throws Exception {
        // Arrange
        String escapedPayload = "He said \\\"hi\\\"\\n";
        String unescapedPayload = "He said \"hi\"\n";
        interact(() -> payloadCodeArea.replaceText(escapedPayload));

        // Act
        invokeControllerMethod(publishViewController, "onUnescape");

        // Assert
        assertEquals(unescapedPayload, payloadCodeArea.getText());
    }

    @Test
    void shouldPreserveBackslashesInUnescapeRoundTrip() throws Exception {
        // Arrange
        String originalPayload = "path\\to\\file";
        interact(() -> payloadCodeArea.replaceText(originalPayload));

        // Act
        invokeControllerMethod(publishViewController, "onEscape");
        invokeControllerMethod(publishViewController, "onUnescape");

        // Assert
        assertEquals(originalPayload, payloadCodeArea.getText());
    }

    @Test
    void shouldApplyPrettifiedTextToPayloadCodeArea() throws Exception {
        // Arrange
        String originalPayload = "{\"foo\":\"bar\"}";
        String prettifiedPayload = "{\n  \"foo\": \"bar\"\n}";
        TestFormat format = new TestFormat(prettifiedPayload, originalPayload, prettifiedPayload);
        setField(publishViewController, "currentFormat", format);
        setField(publishViewController, "autoFormatPayload", new FixedFormatAutoFormatPayload(format));
        interact(() -> payloadCodeArea.replaceText(originalPayload));

        // Act
        invokeControllerMethod(publishViewController, "onPrettify");

        // Assert
        assertEquals(prettifiedPayload, payloadCodeArea.getText());
    }

    @Test
    void shouldApplyMinifiedTextToPayloadCodeArea() throws Exception {
        // Arrange
        String originalPayload = "{\n  \"foo\": \"bar\"\n}";
        String minifiedPayload = "{\"foo\":\"bar\"}";
        setField(publishViewController, "currentFormat", new TestFormat(originalPayload, minifiedPayload, minifiedPayload));
        interact(() -> payloadCodeArea.replaceText(originalPayload));

        // Act
        invokeControllerMethod(publishViewController, "onMinify");

        // Assert
        assertEquals(minifiedPayload, payloadCodeArea.getText());
    }

    @Test
    void shouldNotApplyFixWhenFixedPayloadIsStillInvalid() throws Exception {
        // Arrange
        String originalPayload = "{\"value\": 1}";
        String stillInvalidFixedPayload = "{\"value\": }";
        TestFormat format = new TestFormat(originalPayload, originalPayload, stillInvalidFixedPayload);
        setField(publishViewController, "currentFormat", format);
        setField(publishViewController, "autoFormatPayload", new FixedFormatAutoFormatPayload(format));
        interact(() -> payloadCodeArea.replaceText(originalPayload));

        // Act
        invokeControllerMethod(publishViewController, "onFixJson");

        // Assert
        assertEquals(originalPayload, payloadCodeArea.getText());
    }
    */ // END TODO: Move into plugin UI contributions

    private void setField(Object targetObject, String fieldName, Object fieldValue) throws Exception {
        Class<?> type = targetObject.getClass();
        while (type != null) {
            try {
                Field field = type.getDeclaredField(fieldName);
                field.setAccessible(true);
                field.set(targetObject, fieldValue);
                return;
            } catch (NoSuchFieldException exception) {
                type = type.getSuperclass();
            }
        }
        throw new NoSuchFieldException(fieldName);
    }

    private void invokeControllerMethod(Object targetObject, String methodName) throws Exception {
        Method method = findDeclaredMethod(targetObject.getClass(), methodName);
        method.setAccessible(true);
        interact(() -> {
            try {
                method.invoke(targetObject);
            } catch (Exception exception) {
                throw new IllegalStateException(exception);
            }
        });
    }

    private Method findDeclaredMethod(Class<?> type, String methodName) throws NoSuchMethodException {
        while (type != null) {
            try {
                return type.getDeclaredMethod(methodName);
            } catch (NoSuchMethodException exception) {
                type = type.getSuperclass();
            }
        }
        throw new NoSuchMethodException(methodName);
    }

    private static final class TestFormat implements Format {

        private final String prettifiedPayload;
        private final String minifiedPayload;
        private final String fixedPayload;

        private TestFormat(String prettifiedPayload, String minifiedPayload, String fixedPayload) {
            this.prettifiedPayload = prettifiedPayload;
            this.minifiedPayload = minifiedPayload;
            this.fixedPayload = fixedPayload;
        }

        @Override
        public void setText(String text) {
        }

        @Override
        public boolean isValid() {
            return true;
        }

        @Override
        public String getPrettyString() {
            return prettifiedPayload;
        }

        @Override
        public String getMinifiedString() {
            return minifiedPayload;
        }

        @Override
        public org.fxmisc.richtext.model.StyleSpans<Collection<String>> getFxSpans() {
            Plain plainFormat = new Plain();
            plainFormat.setText(fixedPayload);
            return plainFormat.getFxSpans();
        }

        @Override
        public String getAutoFixedString() {
            return fixedPayload;
        }
    }

    private static final class NoOpAutoFormatPayload extends AutoFormatPayload {

        private NoOpAutoFormatPayload() {
            super(null);
        }

        @Override
        public Format autoFormatPayload(String payload,
                                        boolean doFormatting,
                                        String connectionId,
                                        CodeArea codeArea,
                                        ChangeListener<String> listener,
                                        FormatOptions options) {
            Plain plainFormat = new Plain();
            plainFormat.setText(payload);
            return plainFormat;
        }
    }

    private static final class FixedFormatAutoFormatPayload extends AutoFormatPayload {

        private final Format fixedFormat;

        private FixedFormatAutoFormatPayload(Format fixedFormat) {
            super(null);
            this.fixedFormat = fixedFormat;
        }

        @Override
        public Format autoFormatPayload(String payload,
                                        boolean doFormatting,
                                        String connectionId,
                                        CodeArea codeArea,
                                        ChangeListener<String> listener,
                                        FormatOptions options) {
            fixedFormat.setText(payload);
            return fixedFormat;
        }
    }
}
