package org.correomqtt.business.keyring.userinput;

import org.correomqtt.plugin.spi.KeyringHook;
import org.correomqtt.business.keyring.BaseKeyring;
import org.correomqtt.business.provider.SettingsProvider;
import org.pf4j.Extension;

import java.util.ResourceBundle;

@Extension
public class UserInputKeyring extends BaseKeyring implements KeyringHook {

    private ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale());

    @Override
    public boolean requiresUserinput() {
        return true;
    }

    @Override
    public String getName() {
        return resources.getString("userInputKeyringName");
    }

    @Override
    public String getDescription() {
        return resources.getString("userInputKeyringDescription");
    }

    @Override
    public String getPassword(String label) {
        throw new UnsupportedOperationException();
    }

    @Override
    public void setPassword(String label, String password) {
        throw new UnsupportedOperationException();
    }

    @Override
    public boolean isSupported() {
        return true;
    }

    @Override
    public String getIdentifier() {
        return "UserInput";
    }

    @Override
    public int getSortIndex(){
        return 1;
    }

}
