package org.correomqtt.core.model;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.Setter;

import java.util.EnumMap;
import java.util.List;
import java.util.Map;

@Getter
@Setter
@AllArgsConstructor
@Builder
@JsonInclude(JsonInclude.Include.NON_NULL)
@JsonIgnoreProperties(ignoreUnknown = true)
public class MessageListViewConfig {
    Map<LabelType, Boolean> labelVisibilityMap = new EnumMap<>(LabelType.class);

    public MessageListViewConfig(){
        addLabel(LabelType.RETAINED);
        addLabel(LabelType.QOS);
        addLabel(LabelType.TIMESTAMP);
    }

    public void addLabel(LabelType label) {
        labelVisibilityMap.putIfAbsent(label, false);
    }

    public void addLabels(List<LabelType> labels) {
        labels.forEach(this::addLabel);
    }

    public boolean isVisible(LabelType label) {
        return labelVisibilityMap.get(label) != null && labelVisibilityMap.get(label);
    }

    public void setVisibility(LabelType label, boolean visibility) {
        labelVisibilityMap.put(label, visibility);
    }
}
