package org.correomqtt.business.model;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.Setter;

import java.util.HashMap;
import java.util.List;

@Getter
@Setter
@AllArgsConstructor
@NoArgsConstructor
@Builder
@JsonInclude(JsonInclude.Include.NON_NULL)
@JsonIgnoreProperties(ignoreUnknown = true)
public class MessageListViewConfig {
    HashMap<String, Boolean> labelVisibilityMap = new HashMap<>();

    public void addLabel(String label) {
        labelVisibilityMap.put(label, false);
    }

    public void addLabels(List<String> labels) {
        labels.forEach(this::addLabel);
    }

    public boolean isVisible(String label) {
        return labelVisibilityMap.get(label) != null ? labelVisibilityMap.get(label) : false;
    }

    public void toggleLabel(String label, boolean visibility) {
        labelVisibilityMap.put(label, visibility);
    }
}
