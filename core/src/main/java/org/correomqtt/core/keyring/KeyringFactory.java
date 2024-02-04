package org.correomqtt.core.keyring;

import org.correomqtt.di.DefaultBean;
import org.correomqtt.core.plugin.PluginManager;
import org.correomqtt.core.plugin.spi.KeyringHook;

import org.correomqtt.di.Inject;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

@DefaultBean
public class KeyringFactory {

    private final PluginManager pluginManager;

    @Inject
    KeyringFactory(PluginManager pluginManager) {
        this.pluginManager = pluginManager;
    }

    public List<Keyring> create() {
        return new ArrayList<>(getSupportedKeyrings());
    }

    public Keyring createKeyringByIdentifier(String identifier) {
        return getSupportedKeyrings()
                .stream()
                .filter(k -> k.getIdentifier().equals(identifier))
                .findFirst()
                .orElse(null);
    }

    private List<Keyring> getAllKeyrings() {
        return pluginManager.getExtensions(KeyringHook.class)
                .stream()
                .sorted(Comparator.comparingInt(Keyring::getSortIndex))
                .map(kh -> (Keyring) kh)
                .toList();
    }

    public List<Keyring> getSupportedKeyrings() {
        return getAllKeyrings().stream()
                .filter(Keyring::isSupported)
                .toList();
    }

}