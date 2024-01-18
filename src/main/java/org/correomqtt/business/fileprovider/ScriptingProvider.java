package org.correomqtt.business.fileprovider;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;
import org.apache.commons.io.FileUtils;
import org.correomqtt.business.scripting.ExecutionDTO;
import org.correomqtt.business.scripting.ScriptExecutionError;
import org.correomqtt.business.scripting.ScriptFileDTO;
import org.correomqtt.business.scripting.ScriptingBackend;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.io.FileWriter;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.FileAlreadyExistsException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
import java.util.stream.Stream;

import static org.apache.commons.io.FilenameUtils.removeExtension;

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
        return getFromCache(SCRIPT_FOLDER);
    }

    public String getScriptLogFolder(String filename) {
        return getFromCache(SCRIPT_LOG_FOLDER_NAME + File.separator + filename);

    }

    public List<ScriptFileDTO> getScripts() throws IOException {

        LOGGER.info("Read available scripts");

        String scriptFolder = getScriptFolder();
        Path scriptPath = Paths.get(scriptFolder);
        if (!Files.exists(scriptPath)) {
            Files.createDirectory(scriptPath);
        }

        List<ScriptFileDTO> scripts;

        try (Stream<Path> pathStream = Files.walk(scriptPath)) {
            scripts = pathStream
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

        scripts.forEach(script -> {
            ObjectMapper om = new ObjectMapper();
            om.registerModule(new JavaTimeModule());
            String scriptExecutionFolder = getScriptExecutionsDirectory(script.getName());
            try (Stream<Path> pathStream = Files.walk(new File(scriptExecutionFolder).toPath())) {
                pathStream
                        .filter(f -> Files.isRegularFile(f) && f.getFileName().toString().endsWith(".json"))
                        .filter(f -> ScriptingBackend.getExecutionDTO(removeExtension(f.getFileName().toString())) == null)
                        .forEach(f -> {
                            try {
                                ExecutionDTO dto = om.readValue(f.toFile(), ExecutionDTO.class);
                                dto.setScriptFile(script);
                                if (dto.getExecutionTime() == null) {
                                    dto.setExecutionTime(0L);
                                    dto.setError(new ScriptExecutionError(ScriptExecutionError.Type.HOST, "Unfinished"));
                                    dto.setCancelled(true);
                                }
                                ScriptingBackend.putExecutionDTO(dto.getExecutionId(), dto);
                            } catch (IOException e) {
                                LOGGER.error("Unable to load execution file for {}", f.getFileName(), e);
                            }
                        });
            } catch (IOException e) {
                LOGGER.error("Unable to load execution files for folder {}", scriptExecutionFolder);
            }
        });

        return scripts;
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
        File oldExecutionDir = new File(getScriptExecutionsDirectory(oldName, false));
        File newExecutionDir = new File(getScriptExecutionsDirectory(newName, false));
        File oldLogDir = new File(getScriptLogDirectory(oldName, false));
        File newLogDir = new File(getScriptLogDirectory(newName, false));
        if (newFile.exists()) {
            throw new FileAlreadyExistsException(absoluteNewFilename);
        }
        if (newExecutionDir.exists()) {
            throw new FileAlreadyExistsException(newExecutionDir.getAbsolutePath());
        }
        if (newLogDir.exists()) {
            throw new FileAlreadyExistsException(newLogDir.getAbsolutePath());
        }
        if (!oldFile.renameTo(newFile)) {
            throw new FileAlreadyExistsException(absoluteNewFilename);
        }
        if (oldExecutionDir.exists() && !oldExecutionDir.renameTo(newExecutionDir)) {
            throw new FileAlreadyExistsException(newExecutionDir.getAbsolutePath());
        }
        if (oldLogDir.exists() && !oldLogDir.renameTo(newLogDir)) {
            throw new FileAlreadyExistsException(newLogDir.getAbsolutePath());
        }

        return newFile.toPath();
    }


    public String loadScript(ScriptFileDTO scriptFileDTO) throws IOException {
        StringBuilder codeBuilder = new StringBuilder();
        try (Stream<String> lines = Files.lines(scriptFileDTO.getPath(), StandardCharsets.UTF_8)) {
            lines.forEach(s -> codeBuilder.append(s).append("\n"));
            return codeBuilder.toString();
        }
    }

    public void deleteScript(String filename) throws IOException {
        String absoluteFilename = getScriptFolder() + File.separator + filename;
        File file = new File(absoluteFilename);
        Files.delete(file.toPath());
        deleteExecutions(filename);
    }

    public void deleteExecutions(String filename) throws IOException {
        FileUtils.deleteDirectory(new File(getScriptLogFolder(filename)));
        FileUtils.deleteDirectory(new File(getScriptExecutionsDirectory(filename)));
    }

    public void saveScript(ScriptFileDTO scriptFileDTO, String content) throws IOException {

        try (FileWriter fileWriter = new FileWriter(scriptFileDTO.getPath().toFile())) {
            fileWriter.write(content);
        }
    }

    public String getSingleScriptLogPath(String filename, String executionId) {
        return getScriptLogFolder(filename) + File.separator + executionId + ".log";
    }

    public String loadLog(String filename, String executionId) throws IOException {
        String logFolder = getSingleScriptLogPath(filename, executionId);
        StringBuilder codeBuilder = new StringBuilder();
        File file = new File(logFolder);

        if (!file.exists()) {
            return null;
        }

        try (Stream<String> lines = Files.lines(file.toPath(), StandardCharsets.UTF_8)) {
            lines.forEach(s -> codeBuilder.append(s).append("\n"));
            return codeBuilder.toString().strip();
        }
    }

    public void saveExecution(ExecutionDTO dto) throws IOException, InterruptedException {
        ObjectMapper om = new ObjectMapper();
        om.registerModule(new JavaTimeModule());
        om.writeValue(new File(getScriptExecutionsDirectory(dto.getScriptFile().getName()) + File.separator + dto.getExecutionId() + ".json"), dto);
    }
}
