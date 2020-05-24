package org.correomqtt.gui.model;

import org.correomqtt.business.keyring.Keyring;

public class KeyringModel implements GenericCellModel {

    private final Keyring keyring;

    public KeyringModel(Keyring keyring){
        this.keyring = keyring;
    }

    @Override
    public String getLabelTranslationKey() {
        return keyring.getIdentifier();
    }

    public Keyring getKeyring(){
        return keyring;
    }
}
