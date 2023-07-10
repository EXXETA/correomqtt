package org.correomqtt.business.model;

public enum PublishStatus implements GenericTranslatable {
    PUBLISHED("published"),
    SUCCEEDED("succeeded"),
    FAILED("failed");

    private final String labelTranslationKey;

    PublishStatus(String labelTranslationKey) {
        this.labelTranslationKey = labelTranslationKey;
    }

    public String getLabelTranslationKey() {
        return labelTranslationKey;
    }

    @Override
    public String toString() {
        return labelTranslationKey;
    }
}
