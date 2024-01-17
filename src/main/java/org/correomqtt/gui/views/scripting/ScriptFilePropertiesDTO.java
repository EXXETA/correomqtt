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
import org.apache.commons.io.FileUtils;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

@Getter
@AllArgsConstructor
@Builder
public class ScriptFilePropertiesDTO implements Comparable<ScriptFilePropertiesDTO> {

    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptFilePropertiesDTO.class);

    private final StringProperty nameProperty;
    private final ObjectProperty<Path> pathProperty;
    private final StringProperty codeProperty;
    private final BooleanProperty dirtyProperty;

    public static Callback<ScriptFilePropertiesDTO, Observable[]> extractor() {
        return (ScriptFilePropertiesDTO m) -> new Observable[]{
                m.nameProperty,
                m.pathProperty,
                m.codeProperty,
                m.dirtyProperty
        };
    }

    public String getName() {
        return nameProperty.get();
    }

    public Path getPath() {
        return pathProperty.get();
    }

    public String getCode() {
        return codeProperty.get();
    }

    public boolean isDirty(){
        return dirtyProperty.get();
    }

    @Override
    public String toString() {
        return "Script: " + getName();
    }

    @Override
    public int hashCode() {
        return getName().hashCode() + getPath().hashCode();
    }

    public String getHumanReadableSize() {
        try {
            return FileUtils.byteCountToDisplaySize(Files.size(getPath()));
        } catch (IOException e) {
            LOGGER.debug("Error reading filesize. ", e);
            return "N/A";
        }
    }

    public long getCodeLineCount() {
        if (getCode() == null || getCode().isEmpty()) {
            return 0;
        } else {
            return getCode().lines().count();

        }
    }

    @Override
    public boolean equals(Object o) {
        if (!(o instanceof ScriptFilePropertiesDTO other)) {
            return false;
        }
        return getName().equals(other.getName()) &&
                getPath().equals(other.getPath());
    }

    @Override
    public int compareTo(ScriptFilePropertiesDTO other) {
        if (other == null || other.getName() == null) {
            return 1;
        }

        if (getName() == null) {
            return -1;
        }

        return getName().compareTo(other.getName());
    }

    public boolean isLoaded() {
        return codeProperty.get() != null;
    }

    @SuppressWarnings({"FieldCanBeLocal", "unused"})
    public static class ScriptFilePropertiesDTOBuilder {

        private StringProperty nameProperty = new SimpleStringProperty();
        private ObjectProperty<Path> pathProperty = new SimpleObjectProperty<>();
        private StringProperty codeProperty = new SimpleStringProperty();

        private BooleanProperty dirtyProperty = new SimpleBooleanProperty();

        public ScriptFilePropertiesDTOBuilder name(String name) {
            this.nameProperty.set(name);
            return this;
        }

        public ScriptFilePropertiesDTOBuilder path(Path path) {
            this.pathProperty.set(path);
            return this;
        }

        public ScriptFilePropertiesDTOBuilder code(String code) {
            this.codeProperty.set(code);
            return this;
        }

        public ScriptFilePropertiesDTOBuilder dirty(boolean dirty){
            this.dirtyProperty.set(dirty);
            return this;
        }

        public ScriptFilePropertiesDTO build() {
            return new ScriptFilePropertiesDTO(nameProperty, pathProperty, codeProperty, dirtyProperty);
        }
    }
}