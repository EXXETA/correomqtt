package org.correomqtt.business.keyring.libsecret;

import com.sun.jna.Platform;
import org.correomqtt.business.keyring.BaseKeyring;
import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.plugin.spi.KeyringHook;
import org.freedesktop.secret.simple.SimpleCollection;
import org.pf4j.Extension;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.util.Collections;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.ResourceBundle;

@Extension
public class LibSecretKeyring extends BaseKeyring implements KeyringHook {

    private static final Logger LOGGER = LoggerFactory.getLogger(LibSecretKeyring.class);

    private ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale());

    @Override
    public String getPassword(String label) {
        String item = findItem(label);
        if (item == null) {
            return null;
        }
        return getSecret(item);
    }

    @Override
    public void setPassword(String label, String password) {
        String item = findItem(label);
        if (item == null) {
            createItem(label, password);
        } else {
            updateItem(item, label, password);
        }
    }

    @Override
    public boolean isSupported() {
        return Platform.isLinux() && isAvailable();
    }

    @Override
    public String getIdentifier() {
        return resources.getString("libSecretName");
    }

    @Override
    public String getName() {
        return resources.getString("libSecretDescription");
    }

    @Override
    public String getDescription() {
        return null;
    }

    private boolean isAvailable() {
        try (SimpleCollection collection = new SimpleCollection()) {
            return true;
        } catch (IOException e) {
            LOGGER.debug("Try to detect libsecret failed. This is not a real problem.",e);
            return false;
        }
    }

    private void updateItem(String item, String label, String password) {
        try (SimpleCollection collection = new SimpleCollection()) {
            collection.updateItem(item, label, password, Collections.emptyMap());
        } catch (IOException e) {
            LOGGER.error("Could not update item {} with label {} in libsecret", item, label, e);
        }
    }

    private String findItem(String label) {
        try (SimpleCollection collection = new SimpleCollection()) {
            Map<String, String> searchMap = new HashMap<>();
            searchMap.put("xdg:schema", "org.freedesktop.Secret.Generic");
            List<String> items = collection.getItems(searchMap);
            if (items != null) {
                for (String i : items) {
                    if (label.equals(collection.getLabel(i))) {
                        return i;
                    }
                }
            }
        } catch (IOException e) {
            LOGGER.error("Could not search for item with label {} from libsecret", label, e);
        }
        return null;
    }

    private void createItem(String label, String password) {
        try (SimpleCollection collection = new SimpleCollection()) {
            collection.createItem(label, password);
        } catch (IOException e) {
            LOGGER.error("Could not create item with libsecret for label {}", label, e);
        }
    }

    private String getSecret(String item) {
        try (SimpleCollection collection = new SimpleCollection()) {
            return String.valueOf(collection.getSecret(item));
        } catch (IOException e) {
            LOGGER.error("Could not get secret from libsecret path {}", item, e);
        }
        return null;
    }
}
