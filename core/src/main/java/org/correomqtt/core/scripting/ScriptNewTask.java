package org.correomqtt.core.scripting;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import dagger.assisted.AssistedInject;
import org.correomqtt.core.concurrent.NoProgressTask;
import org.correomqtt.core.concurrent.TaskException;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.fileprovider.ScriptingProvider;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.slf4j.MarkerFactory;

import java.io.IOException;
import java.nio.file.FileAlreadyExistsException;
import java.nio.file.Path;

import static org.correomqtt.core.scripting.ScriptNewTask.Error.FILENAME_EMPTY_OR_WRONG_EXTENSION;
import static org.correomqtt.core.scripting.ScriptNewTask.Error.FILENAME_NULL;
import static org.correomqtt.core.scripting.ScriptNewTask.Error.FILE_ALREADY_EXISTS;
import static org.correomqtt.core.scripting.ScriptNewTask.Error.IOERROR;

public class ScriptNewTask extends NoProgressTask<Path, ScriptNewTask.Error> {

    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptNewTask.class);

    public enum Error {
        FILENAME_NULL,
        FILENAME_EMPTY_OR_WRONG_EXTENSION,
        FILE_ALREADY_EXISTS,
        IOERROR
    }

    private final ScriptingProvider scriptingProvider;
    private final String filename;

    @AssistedFactory
    public interface Factory {
        ScriptNewTask create(String filename);
    }

    @AssistedInject
    public ScriptNewTask(ScriptingProvider scriptingProvider,
                         EventBus eventBus,
                         @Assisted String filename) {
        super(eventBus);
        this.scriptingProvider = scriptingProvider;
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

        String ioErrorMsg = "Error creating new script. ";

        boolean alreadyExists;
        try {
            alreadyExists = scriptingProvider.getScripts()
                    .stream()
                    .anyMatch(s -> s.getName().equals(filename));
        } catch (IOException e) {
            LOGGER.debug(ioErrorMsg, e);
            throw new TaskException(IOERROR);
        }

        if (alreadyExists) {
            throw new TaskException(FILE_ALREADY_EXISTS);
        }

        try {
            return scriptingProvider.createScript(filename, "// Start writing javascript code here.");
        } catch (FileAlreadyExistsException e) {
            LOGGER.debug(MarkerFactory.getMarker(filename), ioErrorMsg, e);
            throw new TaskException(FILE_ALREADY_EXISTS);
        } catch (IOException e) {
            LOGGER.debug(MarkerFactory.getMarker(filename), ioErrorMsg, e);
            throw new TaskException(IOERROR);
        }
    }
}
