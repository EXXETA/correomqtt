package com.exxeta.correomqtt.business.utils;

import com.exxeta.correomqtt.business.dispatcher.ApplicationLifecycleObserver;
import com.exxeta.correomqtt.business.model.ConnectionConfigDTO;
import com.exxeta.correomqtt.business.mqtt.CorreoMqttClient;
import com.exxeta.correomqtt.business.services.ConfigService;
import com.exxeta.correomqtt.business.services.DisconnectService;

import java.util.Comparator;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;
import java.util.stream.Collectors;

public class ConnectionHolder implements ApplicationLifecycleObserver {

    private static ConnectionHolder instance;
    private final Map<String /* connectionId*/, CorreoMqttConnection> connectionMap = new ConcurrentHashMap<>();

    private ConnectionHolder() {
        //private constructor
    }

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

        for ( ConnectionConfigDTO c : ConfigService.getInstance().getConnectionConfigs() ) {
            CorreoMqttConnection connection = connectionMap.get(c.getId());
            if(connection == null){
                connectionMap.put(c.getId(), CorreoMqttConnection.builder()
                                                                 .configDTO(c)
                                                                 .sort(sort)
                                                                 .build());
            }else{
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
        return getConnection(config.getId()).getClient() == null;
    }

    public Map<String, CorreoMqttConnection> getConnections() {
        return connectionMap;
    }

    @Override
    public void onShutdown() {
        connectionMap.keySet().forEach(connectionId -> new DisconnectService(connectionId).disconnect());
    }

    public List<ConnectionConfigDTO> getSortedConnections() {
        return connectionMap.values()
                            .stream()
                            .sorted(Comparator.comparing(CorreoMqttConnection::getSort))
                            .map(CorreoMqttConnection::getConfigDTO)
                            .collect(Collectors.toList());
    }
}