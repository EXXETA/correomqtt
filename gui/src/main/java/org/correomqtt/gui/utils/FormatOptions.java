package org.correomqtt.gui.utils;

public record FormatOptions(
        boolean highlightOnly,
        boolean applyHighlighting,
        boolean showFormatErrors
) {

    public static final FormatOptions DEFAULTS = new FormatOptions(false, true, true);
    public static final FormatOptions HIGHLIGHT_ONLY = new FormatOptions(true, true, true);
    public static final FormatOptions DETECT_ONLY = new FormatOptions(true, false, false);

    public static Builder builder() {
        return new Builder();
    }

    public static final class Builder {

        private boolean highlightOnly;
        private boolean applyHighlighting = true;
        private boolean showFormatErrors = true;

        private Builder() {
        }

        public Builder highlightOnly(boolean highlightOnly) {
            this.highlightOnly = highlightOnly;
            return this;
        }

        public Builder applyHighlighting(boolean applyHighlighting) {
            this.applyHighlighting = applyHighlighting;
            return this;
        }

        public Builder showFormatErrors(boolean showFormatErrors) {
            this.showFormatErrors = showFormatErrors;
            return this;
        }

        public FormatOptions build() {
            return new FormatOptions(highlightOnly, applyHighlighting, showFormatErrors);
        }
    }
}
