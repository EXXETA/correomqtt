package org.correomqtt.gui.model;

import org.correomqtt.core.keyring.Keyring;
import org.correomqtt.core.model.GenericTranslatable;

public class KeyringModel implements GenericTranslatable {

    private final Keyring keyring;

    public KeyringModel(Keyring keyring){
        this.keyring = keyring;
    }

    @Override
    public String getLabelTranslationKey() {
        return keyring.getName();
    }

    public Keyring getKeyring(){
        return keyring;
    }
}
