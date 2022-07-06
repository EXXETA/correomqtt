package org.correomqtt.business.model;


import com.fasterxml.jackson.annotation.JsonView;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.Setter;

import java.util.List;

@Getter
@Setter
@NoArgsConstructor
public class ConnectionExportDTO {
    private String encryptionType;
    private String encryptedData;
    private List<ConnectionConfigDTO> connectionConfigDTOS;


    public ConnectionExportDTO(String encryptionType, String encryptedData) {
        this.encryptionType = encryptionType;
        this.encryptedData = encryptedData;
    }

    public ConnectionExportDTO(List<ConnectionConfigDTO> connectionConfigDTOS) {
        this.connectionConfigDTOS = connectionConfigDTOS;
    }

}
