package com.exxeta.correomqtt.business.model;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
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

    private String id;
    private String name;
    private String url;
    @Builder.Default
    private int port = 1883;
    private String clientId;
    private String username;
    private String password;
    private boolean cleanSession;
    @Builder.Default
    private CorreoMqttVersion mqttVersion = CorreoMqttVersion.MQTT_3_1_1;
    @Builder.Default
    private TlsSsl ssl = TlsSsl.OFF;
    private String sslKeystore;
    private String sslKeystorePassword;
    @Builder.Default
    private Proxy proxy = Proxy.OFF;
    private String sshHost;
    @Builder.Default
    private int sshPort = 22;
    private int localPort;
    @Builder.Default
    private Auth auth = Auth.OFF;
    private String authUsername;
    private String authPassword;
    private String authKeyfile;
    @Builder.Default
    private Lwt lwt = Lwt.OFF;
    private String lwtTopic;
    private Qos lwtQoS;
    private boolean lwtMessageId;
    private boolean lwtAnswerExpected;
    private boolean lwtRetained;
    private String lwtPayload;

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
    }

    public String getHostAndPort() {
        if (getProxy().equals(Proxy.SSH)) {
            return "via " + getSshHost() + ":" + getSshPort();
        } else {
            return getUrl() + ":" + getPort();
        }
    }
}
