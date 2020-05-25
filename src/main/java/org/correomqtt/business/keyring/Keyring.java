package org.correomqtt.business.keyring;

public interface Keyring {

    String getPassword(String label);

    void setPassword(String label, String password);

    boolean isSupported();

    String getIdentifier();

    default boolean requiresUserinput() {
        return false;
    }

    String getName();

    String getDescription();

    default int getSortIndex(){
        return 0;
    }
}
