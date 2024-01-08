package org.correomqtt.business.scripting;

import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.exception.CorreoMqttExportMessageException;
import org.graalvm.polyglot.Context;
import org.graalvm.polyglot.PolyglotException;
import org.graalvm.polyglot.Value;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.io.FileWriter;
import java.io.IOException;
import java.io.PipedInputStream;
import java.io.PipedOutputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.FileAlreadyExistsException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.LocalDateTime;
import java.time.temporal.ChronoUnit;
import java.time.temporal.TemporalUnit;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionException;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.function.Consumer;
import java.util.stream.Stream;

public class ScriptingBackend {
    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptingBackend.class);

    private static ScriptingBackend instance = null;
    private final Map<String, ExecutionContextDTO> executions = new HashMap<>();

    public static synchronized ScriptingBackend getInstance() {
        if (instance == null) {
            instance = new ScriptingBackend();
            return instance;
        } else {
            return instance;
        }
    }

    public static String loadScript(ScriptFileDTO scriptFileDTO) throws IOException {
        StringBuilder codeBuilder = new StringBuilder();
        try (Stream<String> lines = Files.lines(scriptFileDTO.getPath(), StandardCharsets.UTF_8)) {
            lines.forEach(s -> codeBuilder.append(s).append("\n"));
            return codeBuilder.toString();
        }
    }

    ExecutionContextDTO getExecutionContextDTO(String executionId){
        return executions.get(executionId);
    }



    public List<ExecutionDTO> getExecutions() {
        return executions.values()
                .stream()
                .map(ExecutionContextDTO::getExecutionDTO)
                .toList();
    }


    public void putExecution(String executionId, ExecutionContextDTO dto) {
        executions.put(executionId, dto);
    }

}
