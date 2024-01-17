package org.correomqtt.gui.views.scripting;

import javafx.beans.Observable;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.beans.property.SimpleStringProperty;
import javafx.beans.property.StringProperty;
import javafx.util.Callback;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import org.correomqtt.business.scripting.ScriptExecutionError;
import org.correomqtt.business.utils.MessageDateTimeFormatter;

import java.time.LocalDateTime;
import java.time.ZoneOffset;
import java.time.temporal.ChronoUnit;

import static org.correomqtt.gui.views.scripting.ScriptState.CANCELLED;
import static org.correomqtt.gui.views.scripting.ScriptState.FAILED;
import static org.correomqtt.gui.views.scripting.ScriptState.NOTSTARTED;
import static org.correomqtt.gui.views.scripting.ScriptState.RUNNING;
import static org.correomqtt.gui.views.scripting.ScriptState.SUCCEEDED;

@Getter
@AllArgsConstructor
@Builder
public class ExecutionPropertiesDTO implements Comparable<ExecutionPropertiesDTO> {

    private final StringProperty executionIdProperty;
    private final StringProperty connectionIdProperty;
    private final ObjectProperty<ScriptFilePropertiesDTO> scriptFileProperty;
    private final StringProperty jsCodeProperty;
    private final ObjectProperty<ScriptExecutionError> errorProperty;
    private final ObjectProperty<LocalDateTime> startTimeProperty;
    private final ObjectProperty<Long> executionTimeProperty;
    private final BooleanProperty cancelledProperty;

    public static Callback<ExecutionPropertiesDTO, Observable[]> extractor() {
        return (ExecutionPropertiesDTO m) -> new Observable[]{
                m.executionIdProperty,
                m.connectionIdProperty,
                m.scriptFileProperty,
                m.jsCodeProperty,
                m.errorProperty,
                m.startTimeProperty,
                m.executionTimeProperty,
                m.cancelledProperty
        };
    }

    public String getExecutionId() {
        return executionIdProperty.get();
    }

    public String getConnectionId() {
        return connectionIdProperty.get();
    }

    public ScriptFilePropertiesDTO getScriptFilePropertiesDTO() {
        return scriptFileProperty.get();
    }

    public String getJsCode() {
        return jsCodeProperty.get();
    }

    public ScriptExecutionError getError() {
        return errorProperty.get();
    }

    @MessageDateTimeFormatter
    public LocalDateTime getStartTime() {
        return startTimeProperty.get();
    }

    public Long getExecutionTime() {
        return executionTimeProperty.get();
    }

    public boolean isCancelled() {
        return cancelledProperty.get();
    }

    public ScriptState getState() {
        if (this.getStartTime() == null) {
            return NOTSTARTED;
        }

        if (this.getExecutionTime() == null) {
            return RUNNING;
        }

        if (this.getError() == null) {
            return SUCCEEDED;
        }

        if (this.isCancelled()) {
            return CANCELLED;
        }

        return FAILED;
    }

    public LocalDateTime getSortTime() {
        if (getStartTime() == null) {
            return LocalDateTime.ofEpochSecond(0, 0, ZoneOffset.UTC);
        } else if (getExecutionTime() == null) {
            return LocalDateTime.now();
        } else {
            return getStartTime().plus(getExecutionTime(), ChronoUnit.MILLIS);
        }
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
        if (!(o instanceof ExecutionPropertiesDTO other)) {
            return false;
        }
        return getExecutionId().equals(other.getExecutionId());
    }

    @Override
    public int compareTo(ExecutionPropertiesDTO other) {
        if (other == null || other.getExecutionId() == null) {
            return 1;
        }

        if (getExecutionId() == null) {
            return -1;
        }

        return getExecutionId().compareTo(other.getExecutionId());
    }

    @SuppressWarnings({"FieldCanBeLocal", "unused"})
    public static class ExecutionPropertiesDTOBuilder {

        private StringProperty executionIdProperty = new SimpleStringProperty();
        private StringProperty connectionIdProperty = new SimpleStringProperty();
        private ObjectProperty<ScriptFilePropertiesDTO> scriptFileProperty = new SimpleObjectProperty<>();
        private StringProperty jsCodeProperty = new SimpleStringProperty();
        private ObjectProperty<ScriptExecutionError> errorProperty = new SimpleObjectProperty<>();
        private ObjectProperty<LocalDateTime> startTimeProperty = new SimpleObjectProperty<>();
        private ObjectProperty<Long> executionTimeProperty = new SimpleObjectProperty<>();

        private BooleanProperty cancelledProperty = new SimpleBooleanProperty();

        public ExecutionPropertiesDTOBuilder executionId(String executionId) {
            this.executionIdProperty.set(executionId);
            return this;
        }


        public ExecutionPropertiesDTOBuilder connectionId(String connectionId) {
            this.connectionIdProperty.set(connectionId);
            return this;
        }

        public ExecutionPropertiesDTOBuilder scriptFile(ScriptFilePropertiesDTO scriptFileProperty) {
            this.scriptFileProperty.set(scriptFileProperty);
            return this;
        }


        public ExecutionPropertiesDTOBuilder jsCode(String jsCode) {
            this.jsCodeProperty.set(jsCode);
            return this;
        }

        public ExecutionPropertiesDTOBuilder error(ScriptExecutionError error) {
            this.errorProperty.set(error);
            return this;
        }

        public ExecutionPropertiesDTOBuilder startTime(LocalDateTime startTime) {
            this.startTimeProperty.set(startTime);
            return this;
        }

        public ExecutionPropertiesDTOBuilder executionTime(Long executionTime) {
            this.executionTimeProperty.set(executionTime);
            return this;
        }

        public ExecutionPropertiesDTOBuilder cancelled(boolean cancelled) {
            this.cancelledProperty.set(cancelled);
            return this;
        }

        public ExecutionPropertiesDTO build() {
            return new ExecutionPropertiesDTO(executionIdProperty,
                    connectionIdProperty,
                    scriptFileProperty,
                    jsCodeProperty,
                    errorProperty,
                    startTimeProperty,
                    executionTimeProperty,
                    cancelledProperty);
        }
    }
}