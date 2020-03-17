package com.exxeta.correomqtt.business.model;

public enum PublishStatus {
    PUBLISEHD("published"),
    SUCCEEDED("succeeded"),
    FAILED("failed");

    private final String publishStatus;

    PublishStatus(String publishStatus) {
        this.publishStatus = publishStatus;
    }

    public String getPublishStatus() {
        return publishStatus;
    }

    @Override
    public String toString() {
        return publishStatus;
    }
}
