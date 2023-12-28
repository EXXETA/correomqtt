package org.correomqtt.gui.business;

import lombok.extern.slf4j.Slf4j;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.services.ExportConnectionsService;
import org.correomqtt.business.services.ImportConnectionsFileService;
import org.correomqtt.business.services.ImportDecryptConnectionsService;

import java.io.File;
import java.util.List;

@Slf4j
public class ExportTaskFactory {

    private ExportTaskFactory() {
        // private constructor
    }

    public static void exportConnections(File file, List<ConnectionConfigDTO> connectionList, String password) {
        new GuiService<>(new ExportConnectionsService(file, connectionList, password),
                ExportConnectionsService::exportConnections).start();
    }

    public static void importConnectionsFile(File file) {
        new GuiService<>(new ImportConnectionsFileService(file),
                ImportConnectionsFileService::importConnections).start();
    }

    public static void decryptConnections(String encryptedData, String encryptionType, String password){
        new GuiService<>(new ImportDecryptConnectionsService(encryptedData,encryptionType, password),
                ImportDecryptConnectionsService::decrypt).start();
    }
}
