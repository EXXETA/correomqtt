package org.correomqtt.gui.formats;

import org.fxmisc.richtext.model.StyleSpans;

import java.util.Collection;

public interface Format {

    void setText(String text);

    boolean isValid();

    String getPrettyString();

    StyleSpans<Collection<String>> getFxSpans();

    default boolean isFormatable(){
        return true;
    }
}
