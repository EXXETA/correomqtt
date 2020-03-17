package org.correomqtt.business.dispatcher;

public enum LogInfo {

    INFO("Info:\t"),
    ERROR("Error:"),
    SUCCESS("Success:");

    private final String loglvl;

    LogInfo(String loglvl) {
        this.loglvl = loglvl;
    }

    public String getLoglvl() {
        return loglvl;
    }

}
