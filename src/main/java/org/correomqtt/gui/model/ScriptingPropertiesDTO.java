package org.correomqtt.gui.model;

import javafx.beans.Observable;
import javafx.beans.property.*;
import javafx.util.Callback;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;

import java.nio.file.Path;
import java.time.LocalDateTime;

@Getter
@AllArgsConstructor
@Builder
public class ScriptingPropertiesDTO implements Comparable<ScriptingPropertiesDTO> {

    private final StringProperty nameProperty;
    private final ObjectProperty<Path> pathProperty;
    private final StringProperty codeProperty;

    public static Callback<ScriptingPropertiesDTO, Observable[]> extractor() {
        return (ScriptingPropertiesDTO m) -> new Observable[]{
                m.nameProperty,
                m.pathProperty,
                m.codeProperty
        };
    }

    public String getName(){
        return nameProperty.get();
    }

    public Path getPath(){
        return pathProperty.get();
    }

    public String getCode() {
        return codeProperty.get();
    }

    @Override
    public String toString() {
        return "Script: " + getName();
    }

    @Override
    public int hashCode() {
        return getName().hashCode();
    }

    @Override
    public boolean equals(Object o) {
        if (!(o instanceof ScriptingPropertiesDTO)) {
            return false;
        }
        ScriptingPropertiesDTO other = (ScriptingPropertiesDTO) o;
        return ((other.getPath().equals(((ScriptingPropertiesDTO) o).getPath())));
    }

    @Override
    public int compareTo(ScriptingPropertiesDTO other) {
        if (other == null || other.getName() == null) {
            return 1;
        }

        if (getName() == null) {
            return -1;
        }

        return other.getName().compareTo(other.getName());
    }

    @SuppressWarnings({"FieldCanBeLocal", "unused"})
    public static class ScriptingPropertiesDTOBuilder {

        private StringProperty nameProperty = new SimpleStringProperty();
        private ObjectProperty<Path> pathProperty = new SimpleObjectProperty<>();
        private StringProperty codeProperty = new SimpleStringProperty();

        public ScriptingPropertiesDTOBuilder name(String name) {
            this.nameProperty.set(name);
            return this;
        }

        public ScriptingPropertiesDTOBuilder path(Path path) {
            this.pathProperty.set(path);
            return this;
        }

        public ScriptingPropertiesDTOBuilder code(String code) {
            this.codeProperty.set(code);
            return this;
        }

        public ScriptingPropertiesDTO build() {
            return new ScriptingPropertiesDTO(nameProperty, pathProperty, codeProperty);
        }
    }
}