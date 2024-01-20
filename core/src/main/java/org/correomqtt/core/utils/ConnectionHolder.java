package org.correomqtt.core.utils;

import org.correomqtt.core.model.ConnectionConfigDTO;
import org.correomqtt.core.mqtt.CorreoMqttClient;
import org.correomqtt.core.settings.CoreSettings;

import javax.inject.Inject;
import javax.inject.Singleton;
import java.util.Comparator;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;

@Singleton
public class ConnectionHolder {

    private final Map<String /* connectionId*/, CorreoMqttConnection> connectionMap = new ConcurrentHashMap<>();

    private final CoreSettings settings;

    @Inject
    public ConnectionHolder(CoreSettings settings) {
        this.settings = settings;
        refresh();
    }

    public void refresh() {
        int sort = 0;

        Set<String> existingConnectionIds = new HashSet<>(connectionMap.keySet());

        for (ConnectionConfigDTO c : settings.getConnectionConfigs()) {
            CorreoMqttConnection connection = connectionMap.get(c.getId());
            if (connection == null) {
                connectionMap.put(c.getId(), CorreoMqttConnection.builder()
                        .configDTO(c)
                        .sort(sort)
                        .build());
            } else {
                existingConnectionIds.remove(c.getId());
                connection.setConfigDTO(c);
                connection.setSort(sort);
            }
            sort++;
        }

        existingConnectionIds.forEach(connectionMap::remove);
    }

    public CorreoMqttConnection getConnection(String connectionId) {
        return connectionMap.get(connectionId);
    }

    public CorreoMqttClient getClient(String connectionId) {
        CorreoMqttConnection clientConnection = connectionMap.get(connectionId);
        if (clientConnection == null) {
            return null;
        }
        return clientConnection.getClient();
    }

    public ConnectionConfigDTO getConfig(String connectionId) {
        CorreoMqttConnection clientConnection = connectionMap.get(connectionId);
        if (clientConnection == null) {
            return null;
        }
        return clientConnection.getConfigDTO();
    }

    public boolean isConnectionUnused(ConnectionConfigDTO config) {
        CorreoMqttConnection connection = getConnection(config.getId());
        return connection == null || connection.getClient() == null;
    }

    public Map<String, CorreoMqttConnection> getConnections() {
        return connectionMap;
    }

    public List<ConnectionConfigDTO> getSortedConnections() {
        return connectionMap.values()
                .stream()
                .sorted(Comparator.comparing(CorreoMqttConnection::getSort))
                .map(CorreoMqttConnection::getConfigDTO)
                .toList();
    }
}