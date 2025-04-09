package org.correomqtt.core.scripting;

import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.core.concurrent.NoProgressTask;
import org.correomqtt.core.concurrent.TaskException;
import org.correomqtt.di.SoyEvents;
import org.correomqtt.core.fileprovider.ScriptingProvider;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.slf4j.MarkerFactory;

import java.io.IOException;
import java.nio.file.FileAlreadyExistsException;
import java.nio.file.Path;

import static org.correomqtt.core.scripting.ScriptRenameTask.Error.FILENAME_EMPTY_OR_WRONG_EXTENSION;
import static org.correomqtt.core.scripting.ScriptRenameTask.Error.FILENAME_NOT_CHANGED;
import static org.correomqtt.core.scripting.ScriptRenameTask.Error.FILENAME_NULL;
import static org.correomqtt.core.scripting.ScriptRenameTask.Error.FILE_ALREADY_EXISTS;
import static org.correomqtt.core.scripting.ScriptRenameTask.Error.IOERROR;


@DefaultBean
public class ScriptRenameTask extends NoProgressTask<Path, ScriptRenameTask.Error> {

    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptRenameTask.class);

    public enum Error {
        FILENAME_NULL,
        FILENAME_EMPTY_OR_WRONG_EXTENSION,
        FILE_ALREADY_EXISTS,
        FILENAME_NOT_CHANGED,
        IOERROR
    }

    private final ScriptingProvider scriptingProvider;
    private final ScriptFileDTO dto;
    private final String filename;



    @Inject
    public ScriptRenameTask(ScriptingProvider scriptingProvider,
                            SoyEvents soyEvents,
                            @Assisted ScriptFileDTO dto,
                            @Assisted String filename) {
        super(soyEvents);
        this.scriptingProvider = scriptingProvider;
        this.dto = dto;
        this.filename = filename;
    }

    @Override
    protected Path execute() {

        if (filename == null) {
            throw new TaskException(FILENAME_NULL);
        }

        if (filename.length() < 4 || !filename.endsWith(".js")) {
            throw new TaskException(FILENAME_EMPTY_OR_WRONG_EXTENSION);
        }

        if (filename.equals(dto.getName())) {
            throw new TaskException(FILENAME_NOT_CHANGED);
        }

        String ioErrorMsg = "Error renaming script. ";

        boolean alreadyExists;
        try {
            alreadyExists = scriptingProvider.getScripts()
                    .stream()
                    .anyMatch(s -> s.getName().equals(filename));
        } catch (IOException e) {
            LOGGER.error(ioErrorMsg, e);
            throw new TaskException(IOERROR);
        }

        if (alreadyExists) {
            LOGGER.error("Script already exists.");
            throw new TaskException(FILE_ALREADY_EXISTS);
        }

        try {
            // update existing executions
            ScriptingBackend.getExecutions()
                    .stream()
                    .filter(e -> e.getScriptFile().getName().equals(dto.getName()))
                    .forEach(e -> e.getScriptFile().setName(filename));
            return scriptingProvider.renameScript(dto.getName(), filename);
        } catch (FileAlreadyExistsException e) {
            LOGGER.error(MarkerFactory.getMarker(dto.getName()), ioErrorMsg, e);
            throw new TaskException(FILE_ALREADY_EXISTS);
        }
    }
}
