package org.correomqtt.business.keyring.userinput;

import org.correomqtt.plugin.spi.KeyringHook;
import org.pf4j.Extension;

@Extension
public class UserInputKeyring implements KeyringHook {

    @Override
    public boolean requiresUserinput() {
        return true;
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

}
