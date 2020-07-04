package org.correomqtt.business.model;

public enum ConnectionPasswordType {

    PASSWORD("password"), AUTH_PASSWORD("auth_password"), SSL_KEYSTORE_PASSWORD("ssl_keystore_password");

    private final String label;

    ConnectionPasswordType(String label){
        this.label = label;
    }

    public String getLabel(){
        return label;
    }

}
