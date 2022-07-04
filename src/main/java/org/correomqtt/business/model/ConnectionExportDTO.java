package org.correomqtt.business.model;


import lombok.Getter;
import lombok.Setter;

import java.util.List;

@Getter
@Setter
public class ConnectionExportDTO {
    private String encryptionType;
    private String encryptedData;
    private String connectionConfigDTOS;


    public ConnectionExportDTO(String encryptionType, String encryptedData) {
        this.encryptionType = encryptionType;
        this.encryptedData = encryptedData;
    }

    public ConnectionExportDTO(String connectionConfigDTOS) {
        this.connectionConfigDTOS = connectionConfigDTOS;
    }
}
