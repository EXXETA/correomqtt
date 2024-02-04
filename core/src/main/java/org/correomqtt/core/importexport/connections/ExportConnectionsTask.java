package org.correomqtt.core.importexport.connections;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.core.concurrent.NoProgressTask;
import org.correomqtt.core.concurrent.TaskException;
import org.correomqtt.core.encryption.Encryptor;
import org.correomqtt.core.encryption.EncryptorAesGcm;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.fileprovider.EncryptionRecoverableException;
import org.correomqtt.core.model.ConnectionConfigDTO;
import org.correomqtt.core.model.ConnectionConfigDTOMixin;
import org.correomqtt.core.model.ConnectionExportDTO;

import java.io.File;
import java.io.IOException;
import java.util.List;

@DefaultBean
public class ExportConnectionsTask extends NoProgressTask<Integer, ExportConnectionsTask.Error> {

    public enum Error {
        EMPTY_COLLECTION_LIST,
        EMPTY_PASSWORD,
        FILE_IS_NULL,
        MISSING_FILE_EXTENSION
    }

    private final File file;
    private final List<ConnectionConfigDTO> connectionList;
    private final String password;



    @Inject
    public ExportConnectionsTask(EventBus eventBus,
                                 @Assisted File file,
                                 @Assisted List<ConnectionConfigDTO> connectionList,
                                 @Assisted String password) {
        super(eventBus);
        this.file = file;
        this.connectionList = connectionList;
        this.password = password;
    }

    @Override
    protected Integer execute() throws EncryptionRecoverableException, IOException, TaskException {

        if (file == null) {
            throw new TaskException(Error.FILE_IS_NULL);
        }

        if (connectionList == null || connectionList.isEmpty()) {
            throw new TaskException(Error.EMPTY_COLLECTION_LIST);
        }

        if (file.getName().length() < 5 || !file.getName().endsWith(".cqc")) {
            throw new TaskException(Error.MISSING_FILE_EXTENSION);
        }

        // TODO: positive way selecting stuff to export
        connectionList.forEach(connectionConfigDTO -> connectionConfigDTO.setConnectionUISettings(null));

        ConnectionExportDTO connectionExportDTO;

        if (password == null) {
            connectionExportDTO = new ConnectionExportDTO(connectionList);
        } else {
            // TODO check usage of mixin? What is it here?
            String connectionsJSON = new ObjectMapper().addMixIn(ConnectionConfigDTO.class, ConnectionConfigDTOMixin.class).writeValueAsString(connectionList);
            Encryptor encryptor = new EncryptorAesGcm(password);
            String encryptedData = new EncryptorAesGcm(password).encrypt(connectionsJSON);
            connectionExportDTO = new ConnectionExportDTO(encryptor.getEncryptionTranslation(), encryptedData);
        }
        new ObjectMapper().writeValue(file, connectionExportDTO);

        return connectionList.size();
    }
}
