package org.correomqtt.business.model;

import com.fasterxml.jackson.annotation.*;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.Setter;

@Getter
@Setter
@Builder
@NoArgsConstructor
@AllArgsConstructor
@JsonInclude(JsonInclude.Include.NON_NULL)
@JsonIgnoreProperties(ignoreUnknown = true)
public class ConnectionConfigDTO {

    @JsonView(ExportConnectionView.class)
    private String id;
    @JsonView(ExportConnectionView.class)
    private String name;
    @JsonView(ExportConnectionView.class)
    private String url;
    @JsonView(ExportConnectionView.class)
    @Builder.Default
    private int port = 1883;
    @JsonView(ExportConnectionView.class)
    private String clientId;
    @JsonView(ExportConnectionView.class)
    private String username;
    @JsonView(ExportConnectionView.class)
    @JsonProperty(access = JsonProperty.Access.WRITE_ONLY) // do never save passwords
    private String password;
    @JsonView(ExportConnectionView.class)
    private boolean cleanSession;
    @Builder.Default
    @JsonView(ExportConnectionView.class)
    private CorreoMqttVersion mqttVersion = CorreoMqttVersion.MQTT_3_1_1;
    @Builder.Default
    @JsonView(ExportConnectionView.class)
    private TlsSsl ssl = TlsSsl.OFF;
    @JsonView(ExportConnectionView.class)
    private String sslKeystore;
    @JsonView(ExportConnectionView.class)
    @JsonProperty(access = JsonProperty.Access.WRITE_ONLY) // do never save passwords
    private String sslKeystorePassword;
    @JsonView(ExportConnectionView.class)
    @Builder.Default
    private Proxy proxy = Proxy.OFF;
    @JsonView(ExportConnectionView.class)
    private String sshHost;
    @Builder.Default
    @JsonView(ExportConnectionView.class)
    private int sshPort = 22;
    @JsonView(ExportConnectionView.class)
    private int localPort;
    @Builder.Default
    @JsonView(ExportConnectionView.class)
    private Auth auth = Auth.OFF;
    @JsonView(ExportConnectionView.class)
    private String authUsername;
    @JsonView(ExportConnectionView.class)
    @JsonProperty(access = JsonProperty.Access.WRITE_ONLY) // do never save passwords
    private String authPassword;
    @JsonView(ExportConnectionView.class)
    private String authKeyfile;
    @Builder.Default
    @JsonView(ExportConnectionView.class)
    private Lwt lwt = Lwt.OFF;
    @JsonView(ExportConnectionView.class)
    private String lwtTopic;
    @JsonView(ExportConnectionView.class)
    private Qos lwtQoS;
    @JsonView(ExportConnectionView.class)
    private boolean lwtRetained;
    @JsonView(ExportConnectionView.class)
    private String lwtPayload;
    @Builder.Default
    private ConnectionUISettings connectionUISettings = null;

    public ConnectionConfigDTO(ConnectionConfigDTO configDTO) {
        id = configDTO.id;
        name = configDTO.name;
        url = configDTO.url;
        port = configDTO.port;
        clientId = configDTO.clientId;
        username = configDTO.username;
        password = configDTO.password;
        cleanSession = configDTO.cleanSession;
        mqttVersion = configDTO.mqttVersion;
        ssl = configDTO.ssl;
        sslKeystore = configDTO.sslKeystore;
        sslKeystorePassword = configDTO.sslKeystorePassword;
        proxy = configDTO.proxy;
        sshHost = configDTO.sshHost;
        sshPort = configDTO.sshPort;
        localPort = configDTO.localPort;
        auth = configDTO.auth;
        authPassword = configDTO.authPassword;
        authKeyfile = configDTO.authKeyfile;
        connectionUISettings = configDTO.connectionUISettings;
    }

    public String getHostAndPort() {
        if (getProxy().equals(Proxy.SSH)) {
            return "via " + getSshHost() + ":" + getSshPort();
        } else {
            return getUrl() + ":" + getPort();
        }
    }
}
