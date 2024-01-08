package org.correomqtt.business.importexport.connections;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.business.concurrent.NoProgressTask;
import org.correomqtt.business.model.ConnectionExportDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.io.IOException;

public class ImportConnectionsFileTask extends NoProgressTask<ConnectionExportDTO, ImportConnectionsFileTask.Error> {

    private static final Logger LOGGER = LoggerFactory.getLogger(ImportConnectionsFileTask.class);

    public enum Error {
        FILE_IS_NULL,
        FILE_CAN_NOT_BE_READ_OR_PARSED
    }

    private final File file;

    public ImportConnectionsFileTask(File file) {
        this.file = file;
    }

    @Override
    protected ConnectionExportDTO execute() {

        if (file == null) {
            throw createExpectedException(Error.FILE_IS_NULL);
        }

        LOGGER.info("Start importing connections from file {}.", file.getAbsolutePath());
        try {
            return new ObjectMapper().readerFor(ConnectionExportDTO.class).readValue(file);
        } catch (IOException e) {
            LOGGER.debug("File can not be read or parsed.", e);
            throw createExpectedException(Error.FILE_CAN_NOT_BE_READ_OR_PARSED);
        }
    }
}
