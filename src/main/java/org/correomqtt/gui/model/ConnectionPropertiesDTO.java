package org.correomqtt.gui.model;

import javafx.beans.Observable;
import javafx.beans.property.*;
import javafx.util.Callback;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.Setter;
import org.correomqtt.business.model.Lwt;
import org.correomqtt.business.model.Qos;
import org.correomqtt.business.model.*;

import java.util.HashMap;

@Getter
@Setter
@AllArgsConstructor
@Builder
public class ConnectionPropertiesDTO {

    private final StringProperty idProperty;
    private final StringProperty nameProperty;
    private final StringProperty urlProperty;
    private final IntegerProperty portProperty;
    private final StringProperty clientIdProperty;
    private final StringProperty usernameProperty;
    private final StringProperty passwordProperty;
    private final BooleanProperty cleanSessionProperty;
    private final Property<CorreoMqttVersion> mqttVersionProperty;
    private final Property<TlsSsl> sslProperty;
    private final StringProperty sslKeystoreProperty;
    private final StringProperty sslKeystorePasswordProperty;
    private final Property<Proxy> proxyProperty;
    private final StringProperty sshHostProperty;
    private final IntegerProperty sshPortProperty;
    private final IntegerProperty localPortProperty;
    private final Property<Auth> authProperty;
    private final StringProperty authUsernameProperty;
    private final StringProperty authPasswordProperty;
    private final StringProperty authKeyfileProperty;
    private final Property<Lwt> lwtProperty;
    private final StringProperty lwtTopicProperty;
    private final Property<Qos> lwtQoSProperty;
    private final BooleanProperty lwtRetainedProperty;
    private final StringProperty lwtPayloadProperty;
    private final Property<ConnectionUISettings> connectionUISettingsProperty;
    private final BooleanProperty dirtyProperty;
    private final BooleanProperty newProperty;
    private final BooleanProperty unpersistedProperty;
    private final MapProperty<String, Object> extraProperties;

    public static Callback<ConnectionPropertiesDTO, Observable[]> extractor() {
        return (ConnectionPropertiesDTO c) -> new Observable[]{
                c.idProperty,
                c.nameProperty,
                c.urlProperty,
                c.portProperty,
                c.clientIdProperty,
                c.usernameProperty,
                c.passwordProperty,
                c.cleanSessionProperty,
                c.mqttVersionProperty,
                c.sslProperty,
                c.sslKeystoreProperty,
                c.sslKeystorePasswordProperty,
                c.proxyProperty,
                c.sshHostProperty,
                c.sshPortProperty,
                c.localPortProperty,
                c.authProperty,
                c.authUsernameProperty,
                c.authPasswordProperty,
                c.authKeyfileProperty,
                c.lwtProperty,
                c.lwtTopicProperty,
                c.lwtQoSProperty,
                c.lwtRetainedProperty,
                c.lwtPayloadProperty,
                c.connectionUISettingsProperty,
                c.dirtyProperty,
                c.newProperty,
                c.unpersistedProperty,
                c.extraProperties
        };
    }

    public String getId() {
        return idProperty.getValue();
    }

    public String getName() {
        return nameProperty.getValue();
    }

    public String getUrl() {
        return urlProperty.getValue();
    }

    public Integer getPort() {
        return portProperty.getValue();
    }

    public String getClientId() {
        return clientIdProperty.getValue();
    }

    public String getUsername() {
        return usernameProperty.getValue();
    }

    public String getPassword() {
        return passwordProperty.getValue();
    }

    public boolean isCleanSession() {
        return cleanSessionProperty.getValue();
    }

    public CorreoMqttVersion getMqttVersion() {
        return mqttVersionProperty.getValue();
    }

    public TlsSsl getSsl() {
        return sslProperty.getValue();
    }

    public String getSslKeystore() {
        return sslKeystoreProperty.getValue();
    }

    public String getSslKeystorePassword() {
        return sslKeystorePasswordProperty.getValue();
    }

    public Proxy getProxy() {
        return proxyProperty.getValue();
    }

    public String getSshHost() {
        return sshHostProperty.getValue();
    }

    public Integer getSshPort() {
        return sshPortProperty.getValue();
    }

    public Integer getLocalPort() {
        return localPortProperty.getValue();
    }

    public Auth getAuth() {
        return authProperty.getValue();
    }

    public String getAuthUsername() {
        return authUsernameProperty.getValue();
    }

    public String getAuthPassword() {
        return authPasswordProperty.getValue();
    }

    public String getAuthKeyfile() {
        return authKeyfileProperty.getValue();
    }

    public Lwt getLwt() {
        return lwtProperty.getValue();
    }

    public String getLwtTopic() {
        return lwtTopicProperty.getValue();
    }

    public Qos getLwtQos() {
        return lwtQoSProperty.getValue();
    }

    public boolean isLwtRetained() {
        return lwtRetainedProperty.getValue();
    }

    public String getLwtPayload() {
        return lwtPayloadProperty.getValue();
    }

    public ConnectionUISettings getConnectionUISettings() {
        return connectionUISettingsProperty.getValue();
    }

    public boolean isDirty() {
        return dirtyProperty.get();
    }

    public boolean isNew(){
        return newProperty.get();
    }

    public boolean isUnpersisted() {
        return unpersistedProperty.get();
    }

    public String getHostAndPort() {
        if (getProxy().equals(Proxy.SSH)) {
            return "via " + getSshHost() + ":" + getSshPort();
        } else {
            return getUrl() + ":" + getPort();
        }
    }

    @Override
    public String toString() {
        return (dirtyProperty.get() ? "* " : "") + nameProperty.getValue();
    }

    @SuppressWarnings({"FieldCanBeLocal", "unused"})
    public static class ConnectionPropertiesDTOBuilder {

        private StringProperty idProperty = new SimpleStringProperty();
        private StringProperty nameProperty = new SimpleStringProperty();
        private StringProperty urlProperty = new SimpleStringProperty();
        private StringProperty clientIdProperty = new SimpleStringProperty();
        private IntegerProperty portProperty = new SimpleIntegerProperty();
        private StringProperty usernameProperty = new SimpleStringProperty();
        private StringProperty passwordProperty = new SimpleStringProperty();
        private BooleanProperty cleanSessionProperty = new SimpleBooleanProperty();
        private Property<CorreoMqttVersion> mqttVersionProperty = new SimpleObjectProperty<>();
        private Property<TlsSsl> sslProperty = new SimpleObjectProperty<>();
        private StringProperty sslKeystoreProperty = new SimpleStringProperty();
        private StringProperty sslKeystorePasswordProperty = new SimpleStringProperty();
        private Property<Proxy> proxyProperty = new SimpleObjectProperty<>();
        private StringProperty sshHostProperty = new SimpleStringProperty();
        private IntegerProperty sshPortProperty = new SimpleIntegerProperty();
        private IntegerProperty localPortProperty = new SimpleIntegerProperty();
        private Property<Auth> authProperty = new SimpleObjectProperty<>();
        private StringProperty authUsernameProperty = new SimpleStringProperty();
        private StringProperty authPasswordProperty = new SimpleStringProperty();
        private StringProperty authKeyfileProperty = new SimpleStringProperty();
        private Property<Lwt> lwtProperty = new SimpleObjectProperty<>();
        private StringProperty lwtTopicProperty = new SimpleStringProperty();
        private Property<Qos> lwtQoSProperty = new SimpleObjectProperty<>();
        private BooleanProperty lwtMessageIdProperty = new SimpleBooleanProperty();
        private BooleanProperty lwtAnswerExpectedProperty = new SimpleBooleanProperty();
        private BooleanProperty lwtRetainedProperty = new SimpleBooleanProperty();
        private StringProperty lwtPayloadProperty = new SimpleStringProperty();
        private Property<ConnectionUISettings> connectionUISettingsProperty = new SimpleObjectProperty<>();
        private BooleanProperty dirtyProperty = new SimpleBooleanProperty(false);
        private BooleanProperty newProperty = new SimpleBooleanProperty(false);
        private BooleanProperty unpersistedProperty = new SimpleBooleanProperty(true);
        private SimpleMapProperty<String, Object> extraProperties = new SimpleMapProperty<>();

        public ConnectionPropertiesDTOBuilder id(String id) {
            this.idProperty.set(id);
            return this;
        }

        public ConnectionPropertiesDTOBuilder name(String name) {
            this.nameProperty.set(name);
            return this;
        }

        public ConnectionPropertiesDTOBuilder url(String url) {
            this.urlProperty.set(url);
            return this;
        }

        public ConnectionPropertiesDTOBuilder clientId(String clientId) {
            this.clientIdProperty.set(clientId);
            return this;
        }

        public ConnectionPropertiesDTOBuilder port(Integer port) {
            this.portProperty.set(port);
            return this;
        }

        public ConnectionPropertiesDTOBuilder username(String username) {
            this.usernameProperty.set(username);
            return this;
        }

        public ConnectionPropertiesDTOBuilder password(String password) {
            this.passwordProperty.set(password);
            return this;
        }

        public ConnectionPropertiesDTOBuilder cleanSession(boolean cleanSession) {
            this.cleanSessionProperty.set(cleanSession);
            return this;
        }

        public ConnectionPropertiesDTOBuilder mqttVersion(CorreoMqttVersion mqttVersion) {
            this.mqttVersionProperty.setValue(mqttVersion);
            return this;
        }

        public ConnectionPropertiesDTOBuilder ssl(TlsSsl ssl) {
            this.sslProperty.setValue(ssl);
            return this;
        }

        public ConnectionPropertiesDTOBuilder sslKeystore(String sslKeystore) {
            this.sslKeystoreProperty.set(sslKeystore);
            return this;
        }

        public ConnectionPropertiesDTOBuilder sslKeystorePassword(String sslKeystorePassword) {
            this.sslKeystorePasswordProperty.set(sslKeystorePassword);
            return this;
        }

        public ConnectionPropertiesDTOBuilder proxy(Proxy proxy) {
            this.proxyProperty.setValue(proxy);
            return this;
        }

        public ConnectionPropertiesDTOBuilder sshHost(String sshHost) {
            this.sshHostProperty.set(sshHost);
            return this;
        }

        public ConnectionPropertiesDTOBuilder sshPort(Integer sshPort) {
            this.sshPortProperty.set(sshPort);
            return this;
        }

        public ConnectionPropertiesDTOBuilder localPort(Integer localPort) {
            this.localPortProperty.set(localPort);
            return this;
        }

        public ConnectionPropertiesDTOBuilder auth(Auth auth) {
            this.authProperty.setValue(auth);
            return this;
        }

        public ConnectionPropertiesDTOBuilder authUsername(String authUsername) {
            this.authUsernameProperty.set(authUsername);
            return this;
        }

        public ConnectionPropertiesDTOBuilder authPassword(String authPassword) {
            this.authPasswordProperty.set(authPassword);
            return this;
        }

        public ConnectionPropertiesDTOBuilder authKeyfile(String authKeyfile) {
            this.authKeyfileProperty.set(authKeyfile);
            return this;
        }

        public ConnectionPropertiesDTOBuilder lwt(Lwt lwt) {
            this.lwtProperty.setValue(lwt);
            return this;
        }

        public ConnectionPropertiesDTOBuilder lwtTopic(String lwtTopic) {
            this.lwtTopicProperty.setValue(lwtTopic);
            return this;
        }

        public ConnectionPropertiesDTOBuilder lwtQoS(Qos lwtQoS) {
            this.lwtQoSProperty.setValue(lwtQoS);
            return this;
        }

        public ConnectionPropertiesDTOBuilder lwtMessageId(boolean lwtMessageId) {
            this.lwtMessageIdProperty.setValue(lwtMessageId);
            return this;
        }

        public ConnectionPropertiesDTOBuilder lwtAnswerExpected(boolean lwtAnswerExpected) {
            this.lwtAnswerExpectedProperty.setValue(lwtAnswerExpected);
            return this;
        }

        public ConnectionPropertiesDTOBuilder lwtRetained(boolean lwtRetained) {
            this.lwtRetainedProperty.setValue(lwtRetained);
            return this;
        }

        public ConnectionPropertiesDTOBuilder lwtPayload(String lwtPayload) {
            this.lwtPayloadProperty.setValue(lwtPayload);
            return this;
        }

        public ConnectionPropertiesDTOBuilder connectionUISettings(ConnectionUISettings connectionUISettings) {
            this.connectionUISettingsProperty.setValue(connectionUISettings);
            return this;
        }

        public ConnectionPropertiesDTOBuilder dirty(boolean dirty) {
            this.dirtyProperty.set(dirty);
            return this;
        }

        public ConnectionPropertiesDTOBuilder isNew(boolean isNew){
            this.newProperty.set(isNew);
            return this;
        }

        public ConnectionPropertiesDTOBuilder unpersisted(boolean unpersisted) {
            this.unpersistedProperty.set(unpersisted);
            return this;
        }

        private ConnectionPropertiesDTOBuilder extraProperties(HashMap<String, Object> extraProperties) {
            this.extraProperties.putAll(extraProperties);
            return this;
        }

        public ConnectionPropertiesDTO build() {
            return new ConnectionPropertiesDTO(idProperty,
                    nameProperty,
                    urlProperty,
                    portProperty,
                    clientIdProperty,
                    usernameProperty,
                    passwordProperty,
                    cleanSessionProperty,
                    mqttVersionProperty,
                    sslProperty,
                    sslKeystoreProperty,
                    sslKeystorePasswordProperty,
                    proxyProperty,
                    sshHostProperty,
                    sshPortProperty,
                    localPortProperty,
                    authProperty,
                    authUsernameProperty,
                    authPasswordProperty,
                    authKeyfileProperty,
                    lwtProperty,
                    lwtTopicProperty,
                    lwtQoSProperty,
                    lwtRetainedProperty,
                    lwtPayloadProperty,
                    connectionUISettingsProperty,
                    dirtyProperty,
                    newProperty,
                    unpersistedProperty,
                    extraProperties);
        }
    }
}
