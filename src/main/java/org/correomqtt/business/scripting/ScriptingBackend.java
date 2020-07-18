package org.correomqtt.business.scripting;

import org.correomqtt.business.exception.CorreoMqttExportMessageException;
import org.correomqtt.business.model.ScriptExecutionDTO;
import org.correomqtt.business.model.ScriptingDTO;
import org.graalvm.polyglot.Context;
import org.graalvm.polyglot.Value;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.function.Consumer;

public class ScriptingBackend {

    public static String loadScript(ScriptingDTO scriptingDTO) {
        StringBuilder codeBuilder = new StringBuilder();
        try {
            Files.lines(scriptingDTO.getPath(), StandardCharsets.UTF_8)
                    .forEach(s -> codeBuilder.append(s).append("\n"));
            return codeBuilder.toString();

        } catch (IOException e) {
            throw new CorreoMqttExportMessageException(e); //TODO
        }
    }

    public static long executeScript(ScriptExecutionDTO scriptExecutionDTO) {
        try (Context context = Context.newBuilder("js")
                .out(scriptExecutionDTO.getOut())
                .build()) {
            context.getBindings("js").putMember("correo", new CorreoJsBinding(scriptExecutionDTO));
            long start = System.currentTimeMillis();
            context.eval("js", scriptExecutionDTO.getJsCode());
            long end = System.currentTimeMillis();
            return end - start;
        }

    }
}
