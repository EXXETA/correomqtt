package org.correomqtt.core.keyring.osxkeychain;

import com.github.javakeyring.PasswordAccessException;
import com.github.javakeyring.internal.osx.ModernOsxKeychainBackend;
import org.correomqtt.core.keyring.BaseKeyring;
import org.correomqtt.core.keyring.KeyringException;
import org.correomqtt.core.plugin.spi.KeyringHook;
import org.pf4j.Extension;

@SuppressWarnings("unused")
@Extension
public class OSXKeychainKeyring extends BaseKeyring implements KeyringHook {

    private static final String SERVICE_NAME = "CorreoMQTT";
    public static final String FAILED_TO_RETRIEVE_PASSWORD_FROM_OSX_KEYCHAIN = "Failed to retrieve password from osx keychain.";

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
        return "osxKeychainName";
    }

    @Override
    public String getDescription() {
        return "osxKeychainDescription";
    }
}
