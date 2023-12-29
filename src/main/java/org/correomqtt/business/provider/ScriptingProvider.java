package org.correomqtt.business.provider;

import org.correomqtt.business.model.ScriptingDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
import java.util.stream.Collectors;
import java.util.stream.Stream;

public class ScriptingProvider extends BaseUserFileProvider {

    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptingProvider.class);

    private static ScriptingProvider instance = null;

    private static final String SCRIPT_FOLDER = "scripts";

    public static synchronized ScriptingProvider getInstance() {
        if (instance == null) {
            instance = new ScriptingProvider();
            return instance;
        } else {
            return instance;
        }
    }

    public List<ScriptingDTO> getScripts() throws IOException {

        LOGGER.info("Read available scripts");

        String scriptFolder = getTargetDirectoryPath() + File.separator + SCRIPT_FOLDER;
        Path scriptPath = Paths.get(scriptFolder);
        if (!Files.exists(scriptPath)) {
            Files.createDirectory(scriptPath);
        }

        try (Stream<Path> pathStream = Files.walk(scriptPath)) {
            return pathStream
                    .filter(f -> Files.isRegularFile(f) && f.getFileName().toString().endsWith(".js"))
                    .map(f -> {
                        LOGGER.info("Found script \"{}\"", f.toAbsolutePath());
                        return ScriptingDTO.builder()
                                .name(f.getFileName().toString())
                                .path(f.toAbsolutePath())
                                .build();
                    })
                    .toList();
        }
    }

}
