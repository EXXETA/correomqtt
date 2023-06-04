package org.correomqtt.business.model;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.Setter;

import java.util.HashMap;
import java.util.List;

@Getter
@Setter
@AllArgsConstructor
@Builder
@JsonInclude(JsonInclude.Include.NON_NULL)
@JsonIgnoreProperties(ignoreUnknown = true)
public class MessageListViewConfig {
    HashMap<LabelType, Boolean> labelVisibilityMap = new HashMap<>();

    public MessageListViewConfig(){
        addLabel(LabelType.RETAINED);
        addLabel(LabelType.QOS);
        addLabel(LabelType.TIMESTAMP);
    }

    public void addLabel(LabelType label) {
        if(!labelVisibilityMap.containsKey(label)){
            labelVisibilityMap.put(label, false);
        }
    }

    public void addLabels(List<LabelType> labels) {
        labels.forEach(this::addLabel);
    }

    public boolean isVisible(LabelType label) {
        return labelVisibilityMap.get(label) != null ? labelVisibilityMap.get(label) : false;
    }

    public void setVisibility(LabelType label, boolean visibility) {
        labelVisibilityMap.put(label, visibility);
    }
}
