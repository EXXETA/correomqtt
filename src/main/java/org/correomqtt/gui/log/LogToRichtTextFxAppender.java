package org.correomqtt.gui.log;

import ch.qos.logback.classic.spi.ILoggingEvent;
import ch.qos.logback.core.AppenderBase;
import ch.qos.logback.core.encoder.Encoder;
import javafx.application.Platform;
import lombok.Getter;
import lombok.Setter;
import org.correomqtt.gui.utils.LogAreaUtils;
import org.fxmisc.richtext.StyleClassedTextArea;

import java.nio.charset.StandardCharsets;

public class LogToRichtTextFxAppender extends AppenderBase<ILoggingEvent> {

    @Setter
    @Getter
    protected Encoder<ILoggingEvent> encoder;
    private final StyleClassedTextArea area;

    public LogToRichtTextFxAppender(StyleClassedTextArea area) {
        this.area = area;
    }

    @Override
    public void start() {
        super.start();
        addData(encoder.footerBytes());
    }

    @Override
    protected void append(ILoggingEvent eventObject) {
        addData(encoder.encode(eventObject));
    }

    private void addData(byte[] bytes) {
        String msg = new String(bytes, StandardCharsets.UTF_8);
        Platform.runLater(() ->
                LogAreaUtils.appendColorful(area, msg)
        );
    }

    @Override
    public void stop() {
        addData(encoder.headerBytes());
        super.stop();
    }
}
