package org.correomqtt.gui.formats;

import org.fxmisc.richtext.model.StyleSpans;
import org.fxmisc.richtext.model.StyleSpansBuilder;

import java.util.Collection;
import java.util.Collections;

public class Plain implements Format {

    private String text;

    @Override
    public void setText(String text) {
        this.text = text;
    }

    @Override
    public boolean isValid() {
        return true;
    }

    @Override
    public String getPrettyString() {
        return text;
    }

    @Override
    public StyleSpans<Collection<String>> getFxSpans() {
        StyleSpansBuilder<Collection<String>> spansBuilder = new StyleSpansBuilder<>();
        spansBuilder.add(Collections.emptyList(), getPrettyString().length());
        return spansBuilder.create();
    }

    @Override
    public boolean isFormatable() {
        return false;
    }
}
