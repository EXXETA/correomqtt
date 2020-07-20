package org.correomqtt.gui.model;

import javafx.beans.Observable;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.beans.property.SimpleStringProperty;
import javafx.beans.property.StringProperty;
import javafx.util.Callback;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;

import java.nio.file.Path;

@Getter
@AllArgsConstructor
@Builder
public class ExecutionPropertiesDTO implements Comparable<ExecutionPropertiesDTO> {

    private final StringProperty executionIdProperty;
    private final StringProperty logProperty;

    public static Callback<ExecutionPropertiesDTO, Observable[]> extractor() {
        return (ExecutionPropertiesDTO m) -> new Observable[]{
                m.executionIdProperty
        };
    }

    public String getExecutionId(){
        return executionIdProperty.get();
    }

    public String getLog(){
        return logProperty.get();
    }

    @Override
    public String toString() {
        return "Execution: " + getExecutionId();
    }

    @Override
    public int hashCode() {
        return getExecutionId().hashCode();
    }

    @Override
    public boolean equals(Object o) {
        if (!(o instanceof ExecutionPropertiesDTO)) {
            return false;
        }
        ExecutionPropertiesDTO other = (ExecutionPropertiesDTO) o;
        return ((other.getExecutionId().equals(((ExecutionPropertiesDTO) o).getExecutionId())));
    }

    @Override
    public int compareTo(ExecutionPropertiesDTO other) {
        if (other == null || other.getExecutionId() == null) {
            return 1;
        }

        if (getExecutionId() == null) {
            return -1;
        }

        return other.getExecutionId().compareTo(other.getExecutionId());
    }

    @SuppressWarnings({"FieldCanBeLocal", "unused"})
    public static class ExecutionPropertiesDTOBuilder {

        private StringProperty executionIdProperty = new SimpleStringProperty();
        private StringProperty logProperty = new SimpleStringProperty();

        public ExecutionPropertiesDTOBuilder executionId(String executionId) {
            this.executionIdProperty.set(executionId);
            return this;
        }

        public ExecutionPropertiesDTOBuilder log(String log) {
            this.logProperty.set(log);
            return this;
        }

        public ExecutionPropertiesDTO build() {
            return new ExecutionPropertiesDTO(executionIdProperty, logProperty);
        }
    }
}