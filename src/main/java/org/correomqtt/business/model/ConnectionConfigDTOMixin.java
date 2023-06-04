package org.correomqtt.business.model;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.fasterxml.jackson.annotation.JsonProperty;
import lombok.Builder;

public abstract class ConnectionConfigDTOMixin {

    String id;
    String name;
    String url;
    @Builder.Default
    int port = 1883;
    String clientId;
    String username;
    @JsonProperty(access = JsonProperty.Access.READ_WRITE)
    String password;
    boolean cleanSession;
    @Builder.Default
    CorreoMqttVersion mqttVersion = CorreoMqttVersion.MQTT_3_1_1;
    @Builder.Default
    TlsSsl ssl = TlsSsl.OFF;
    String sslKeystore;
    @JsonProperty(access = JsonProperty.Access.READ_WRITE)
    String sslKeystorePassword;
    @Builder.Default
    Proxy proxy = Proxy.OFF;
    String sshHost;
    @Builder.Default
    int sshPort = 22;
    int localPort;
    @Builder.Default
    Auth auth = Auth.OFF;
    String authUsername;
    @JsonProperty(access = JsonProperty.Access.READ_WRITE)
    String authPassword;
    String authKeyfile;
    @Builder.Default
    Lwt lwt = Lwt.OFF;
    String lwtTopic;
    Qos lwtQoS;
    boolean lwtRetained;
    String lwtPayload;
    @JsonIgnore
    ConnectionUISettings connectionUISettings = null;
}
