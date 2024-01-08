package org.correomqtt.business.scripting;

import org.correomqtt.business.concurrent.NoProgressTask;
import org.correomqtt.business.fileprovider.ScriptingProvider;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.slf4j.MarkerFactory;

import java.io.IOException;
import java.nio.file.FileAlreadyExistsException;
import java.nio.file.Path;

import static org.correomqtt.business.scripting.ScriptNewTask.Error.FILENAME_EMPTY_OR_WRONG_EXTENSION;
import static org.correomqtt.business.scripting.ScriptNewTask.Error.FILENAME_NULL;
import static org.correomqtt.business.scripting.ScriptNewTask.Error.FILE_ALREADY_EXISTS;
import static org.correomqtt.business.scripting.ScriptNewTask.Error.IOERROR;

public class ScriptNewTask extends NoProgressTask<Path, ScriptNewTask.Error> {

    private final Logger LOGGER = LoggerFactory.getLogger(ScriptNewTask.class);

    public enum Error {
        FILENAME_NULL,
        FILENAME_EMPTY_OR_WRONG_EXTENSION,
        FILE_ALREADY_EXISTS,
        IOERROR
    }

    private final String filename;

    public ScriptNewTask(String filename) {
        this.filename = filename;
    }

    @Override
    protected Path execute() {

        if (filename == null) {
            throw createExpectedException(FILENAME_NULL);
        }

        if (filename.length() < 4 || !filename.endsWith(".js")) {
            throw createExpectedException(FILENAME_EMPTY_OR_WRONG_EXTENSION);
        }

        String ioErrorMsg = "Error creating new script. ";

        boolean alreadyExists;
        try {
            alreadyExists = ScriptingProvider.getInstance().getScripts()
                    .stream()
                    .anyMatch(s -> s.getName().equals(filename));
        } catch (IOException e) {
            LOGGER.debug(ioErrorMsg, e);
            throw createExpectedException(IOERROR);
        }

        if (alreadyExists) {
            throw createExpectedException(FILE_ALREADY_EXISTS);
        }

        try {
            return ScriptingProvider.getInstance().createScript(filename, "// Start writing javascript code here.");
        } catch (FileAlreadyExistsException e) {
            LOGGER.debug(MarkerFactory.getMarker(filename),ioErrorMsg, e);
            throw createExpectedException(FILE_ALREADY_EXISTS);
        } catch (IOException e) {
            LOGGER.debug(MarkerFactory.getMarker(filename),ioErrorMsg, e);
            throw createExpectedException(IOERROR);
        }
    }
}
