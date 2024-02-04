package org.correomqtt.core.importexport.connections;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.core.concurrent.NoProgressTask;
import org.correomqtt.core.concurrent.TaskException;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.model.ConnectionExportDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.io.IOException;

@DefaultBean
public class ImportConnectionsFileTask extends NoProgressTask<ConnectionExportDTO, ImportConnectionsFileTask.Error> {

    private static final Logger LOGGER = LoggerFactory.getLogger(ImportConnectionsFileTask.class);

    public enum Error {
        FILE_IS_NULL,
        FILE_CAN_NOT_BE_READ_OR_PARSED
    }

    private final File file;



    @Inject
    public ImportConnectionsFileTask(EventBus eventBus,
                                     @Assisted File file) {
        super(eventBus);
        this.file = file;
    }

    @Override
    protected ConnectionExportDTO execute() throws TaskException {

        if (file == null) {
            throw new TaskException(Error.FILE_IS_NULL);
        }

        LOGGER.info("Start importing connections from file {}.", file.getAbsolutePath());
        try {
            return new ObjectMapper().readerFor(ConnectionExportDTO.class).readValue(file);
        } catch (IOException e) {
            LOGGER.debug("File can not be read or parsed.", e);
            throw new TaskException(Error.FILE_CAN_NOT_BE_READ_OR_PARSED);
        }
    }
}
