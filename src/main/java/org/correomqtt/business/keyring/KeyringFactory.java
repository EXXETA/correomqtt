package org.correomqtt.business.keyring;

import org.correomqtt.plugin.manager.PluginManager;
import org.correomqtt.plugin.spi.KeyringHook;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.stream.Collectors;

public class KeyringFactory {

    public static Keyring create() {
        return getSupportedKeyrings()
                .stream()
                .findFirst()
                .orElseThrow(() -> new KeyringException("No supported keyring backend found."));
    }

    public static Keyring createKeyringByIdentifier(String identifier) {
        return getSupportedKeyrings()
                .stream()
                .filter(k -> k.getIdentifier().equals(identifier))
                .findFirst()
                .orElse(null);
    }

    private static List<Keyring> getAllKeyrings() {
        return PluginManager.getInstance().getExtensions(KeyringHook.class)
                .stream()
                .sorted(Comparator.comparingInt(Keyring::getSortIndex))
                .collect(Collectors.toList());
    }

    public static List<Keyring> getSupportedKeyrings() {
        return getAllKeyrings().stream()
                .filter(Keyring::isSupported)
                .collect(Collectors.toList());
    }

}