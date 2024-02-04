package org.correomqtt.core.keyring;

public abstract class BaseKeyring implements Keyring {

    @Override
    public String toString(){
        return getName();
    }
}
