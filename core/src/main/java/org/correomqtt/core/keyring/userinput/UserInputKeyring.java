package org.correomqtt.core.keyring.userinput;

import org.correomqtt.core.keyring.BaseKeyring;
import org.correomqtt.core.plugin.spi.KeyringHook;
import org.pf4j.Extension;

@SuppressWarnings("unused")
@Extension
public class UserInputKeyring extends BaseKeyring implements KeyringHook {

    @Override
    public boolean requiresUserinput() {
        return true;
    }

    @Override
    public String getName() {
        return "userInputKeyringName";
    }

    @Override
    public String getDescription() {
        return "userInputKeyringDescription";
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
