package com.exxeta.correomqtt.gui.transformer;

import com.exxeta.correomqtt.business.model.ConnectionConfigDTO;
import com.exxeta.correomqtt.gui.model.ConnectionPropertiesDTO;
import javafx.collections.ObservableList;

import java.util.List;
import java.util.stream.Collectors;

public class ConnectionTransformer {

    private ConnectionTransformer() {
        // private constructor
    }

    public static List<ConnectionPropertiesDTO> dtoListToPropList(List<ConnectionConfigDTO> connectionDTOList) {
        return connectionDTOList.stream()
                .map(ConnectionTransformer::dtoToProps)
                .collect(Collectors.toList());
    }

    public static ConnectionPropertiesDTO dtoToProps(ConnectionConfigDTO dto) {
        return ConnectionPropertiesDTO.builder()
                .id(dto.getId())
                .name(dto.getName())
                .url(dto.getUrl())
                .clientId(dto.getClientId())
                .port(dto.getPort())
                .username(dto.getUsername())
                .password(dto.getPassword())
                .cleanSession(dto.isCleanSession())
                .mqttVersion(dto.getMqttVersion())
                .ssl(dto.getSsl())
                .sslKeystore(dto.getSslKeystore())
                .sslKeystorePassword(dto.getSslKeystorePassword())
                .proxy(dto.getProxy())
                .sshHost(dto.getSshHost())
                .sshPort(dto.getSshPort())
                .localPort(dto.getLocalPort())
                .auth(dto.getAuth())
                .authUsername(dto.getAuthUsername())
                .authPassword(dto.getAuthPassword())
                .authKeyfile(dto.getAuthKeyfile())
                .lwt(dto.getLwt())
                .lwtTopic(dto.getLwtTopic())
                .lwtQoS(dto.getLwtQoS())
                .lwtMessageId(dto.isLwtMessageId())
                .lwtAnswerExpected(dto.isLwtAnswerExpected())
                .lwtRetained(dto.isLwtAnswerExpected())
                .lwtPayload(dto.getLwtPayload())
                .dirty(false)
                .unpersisted(false)
                .build();
    }

    public static List<ConnectionConfigDTO> propsListToDtoList(ObservableList<ConnectionPropertiesDTO> connectionPropList) {
        return connectionPropList.stream()
                .map(ConnectionTransformer::propsToDto)
                .collect(Collectors.toList());
    }

    public static ConnectionConfigDTO propsToDto(ConnectionPropertiesDTO props) {
        return ConnectionConfigDTO.builder()
                .id(props.getId())
                .name(props.getName())
                .url(props.getUrl())
                .clientId(props.getClientId())
                .port(props.getPort())
                .username(props.getUsername())
                .password(props.getPassword())
                .cleanSession(props.isCleanSession())
                .mqttVersion(props.getMqttVersion())
                .ssl(props.getSsl())
                .sslKeystore(props.getSslKeystore())
                .sslKeystorePassword(props.getSslKeystorePassword())
                .proxy(props.getProxy())
                .sshHost(props.getSshHost())
                .sshPort(props.getSshPort())
                .localPort(props.getLocalPort())
                .auth(props.getAuth())
                .authUsername(props.getAuthUsername())
                .authPassword(props.getAuthPassword())
                .authKeyfile(props.getAuthKeyfile())
                .lwt(props.getLwt())
                .lwtTopic(props.getLwtTopic())
                .lwtQoS(props.getLwtQos())
                .lwtMessageId(props.getLwtMessageId())
                .lwtAnswerExpected(props.getLwtAnswerExpected())
                .lwtRetained(props.getLwtRetained())
                .lwtPayload(props.getLwtPayload())
                .build();
    }
}
