package org.correomqtt.business.fileprovider;

import org.correomqtt.business.scripting.ScriptFileDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.io.FileNotFoundException;
import java.io.FileWriter;
import java.io.IOException;
import java.nio.file.FileAlreadyExistsException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
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

    private String getScriptFolder() {
        return getTargetDirectoryPath() + File.separator + SCRIPT_FOLDER;
    }

    public List<ScriptFileDTO> getScripts() throws IOException {

        LOGGER.info("Read available scripts");

        String scriptFolder = getScriptFolder();
        Path scriptPath = Paths.get(scriptFolder);
        if (!Files.exists(scriptPath)) {
            Files.createDirectory(scriptPath);
        }

        try (Stream<Path> pathStream = Files.walk(scriptPath)) {
            return pathStream
                    .filter(f -> Files.isRegularFile(f) && f.getFileName().toString().endsWith(".js"))
                    .map(f -> {
                        LOGGER.info("Found script \"{}\"", f.toAbsolutePath());
                        return ScriptFileDTO.builder()
                                .name(f.getFileName().toString())
                                .path(f.toAbsolutePath())
                                .build();
                    })
                    .toList();
        }
    }

    public Path createScript(String filename, String content) throws IOException {
        String absoluteFilename = getScriptFolder() + File.separator + filename;
        File file = new File(absoluteFilename);
        if (!file.createNewFile()) {
            throw new FileAlreadyExistsException(absoluteFilename);
        }

        try (FileWriter fileWriter = new FileWriter(absoluteFilename)) {
            fileWriter.write(content);
        }

        return Path.of(absoluteFilename);
    }

    public Path renameScript(String oldName, String newName) throws FileAlreadyExistsException {
        String absoluteOldFilename = getScriptFolder() + File.separator + oldName;
        String absoluteNewFilename = getScriptFolder() + File.separator + newName;
        File oldFile = new File(absoluteOldFilename);
        File newFile = new File(absoluteNewFilename);
        if (!oldFile.renameTo(newFile)) {
            throw new FileAlreadyExistsException(absoluteNewFilename);
        }
        return newFile.toPath();
    }

    public void deleteScript(String filename) throws IOException {
        String absoluteFilename = getScriptFolder() + File.separator + filename;
        File file = new File(absoluteFilename);
        Files.delete(file.toPath());
    }

    public void saveScript(ScriptFileDTO scriptFileDTO, String content) throws IOException {

        try (FileWriter fileWriter = new FileWriter(scriptFileDTO.getPath().toFile())) {
            fileWriter.write(content);
        }
    }
}
