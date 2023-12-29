package org.correomqtt.business.utils;

import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.provider.SettingsProvider;

import java.util.Comparator;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;

public class ConnectionHolder {

    private static ConnectionHolder instance;
    private final Map<String /* connectionId*/, CorreoMqttConnection> connectionMap = new ConcurrentHashMap<>();

    private ConnectionHolder() {}

    public static synchronized ConnectionHolder getInstance() {
        if (instance == null) {
            instance = new ConnectionHolder();
            instance.refresh();
        }
        return instance;
    }

    public void refresh() {
        int sort = 0;

        Set<String> existingConnectionIds = new HashSet<>(connectionMap.keySet());

        for (ConnectionConfigDTO c : SettingsProvider.getInstance().getConnectionConfigs()) {
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