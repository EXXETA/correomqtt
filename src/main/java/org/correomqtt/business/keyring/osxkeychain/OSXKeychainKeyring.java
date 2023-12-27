package org.correomqtt.business.keyring.osxkeychain;

import com.github.javakeyring.PasswordAccessException;
import com.github.javakeyring.internal.osx.ModernOsxKeychainBackend;
import org.correomqtt.plugin.spi.KeyringHook;
import org.correomqtt.business.keyring.BaseKeyring;
import org.correomqtt.business.keyring.KeyringException;
import org.correomqtt.business.provider.SettingsProvider;
import org.pf4j.Extension;

import java.util.ResourceBundle;

@Extension
public class OSXKeychainKeyring extends BaseKeyring implements KeyringHook {

    private static final String SERVICE_NAME = "CorreoMQTT";
    public static final String FAILED_TO_RETRIEVE_PASSWORD_FROM_OSX_KEYCHAIN = "Failed to retrieve password from osx keychain.";

    private ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale());

    @Override
    public String getPassword(String label) {
        try(ModernOsxKeychainBackend keychainBackend = new ModernOsxKeychainBackend()) {
            return keychainBackend.getPassword(SERVICE_NAME, label);
        }catch(PasswordAccessException e){
            return null;
        } catch (Exception e) {
            throw new KeyringException(FAILED_TO_RETRIEVE_PASSWORD_FROM_OSX_KEYCHAIN, e);
        }
    }

    @Override
    public void setPassword(String label, String password) {
        try(ModernOsxKeychainBackend keychainBackend = new ModernOsxKeychainBackend()) {
            keychainBackend.setPassword(SERVICE_NAME, label, password);
        } catch (Exception e) {
            throw new KeyringException(FAILED_TO_RETRIEVE_PASSWORD_FROM_OSX_KEYCHAIN, e);
        }
    }

    @Override
    public boolean isSupported() {

        try(ModernOsxKeychainBackend ignored = new ModernOsxKeychainBackend()) {
            return true;
        } catch (Exception e) {
            return false;
        }
    }

    @Override
    public String getIdentifier() {
        return "OSXKeychain";
    }

    @Override
    public String getName() {
        return resources.getString("osxKeychainName");
    }

    @Override
    public String getDescription() {
        return resources.getString("osxKeychainDescription");
    }
}
