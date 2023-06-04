package org.correomqtt.gui.model;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.Setter;
import org.apache.commons.lang3.StringUtils;
import org.correomqtt.business.model.GenericTranslatable;

import java.util.Locale;

@Getter
@Setter
@AllArgsConstructor
@Builder
public class LanguageModel implements GenericTranslatable {

    private Locale locale;

    @Override
    public String getLabelTranslationKey() {
        return StringUtils.capitalize(locale.getDisplayLanguage(locale));
    }

    @Override
    public boolean equals(Object language){
        if(!(language instanceof LanguageModel)){
            return false;
        }

        return ((LanguageModel) language).getLocale().equals(locale);
    }

    @Override
    public int hashCode(){
        return locale.hashCode();
    }

}
