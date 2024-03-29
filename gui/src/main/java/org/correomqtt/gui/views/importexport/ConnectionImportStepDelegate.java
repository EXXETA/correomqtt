package org.correomqtt.gui.views.importexport;

import org.correomqtt.core.model.ConnectionConfigDTO;
import org.correomqtt.core.model.ConnectionExportDTO;

import java.util.List;

public interface ConnectionImportStepDelegate {
    void setOriginalImportedDTO(ConnectionExportDTO originalImportedDTO);

    void goStepDecrypt();

    void goStepConnections();

    void goStepChooseFile();

    void goStepFinal();

    void onCancelClicked();

    List<ConnectionConfigDTO> getOriginalImportedConnections();

    ConnectionExportDTO getOriginalImportedDTO();

    void setOriginalImportedConnections(List<ConnectionConfigDTO> connectionList);

    void setImportableConnections(List<ConnectionConfigDTO> connectionList);

    List<ConnectionConfigDTO> getImportableConnections();

}
