package org.correomqtt.gui.model;

import javafx.beans.Observable;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.ListProperty;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.beans.property.SimpleListProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.beans.property.SimpleStringProperty;
import javafx.beans.property.StringProperty;
import javafx.collections.ObservableList;
import javafx.util.Callback;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.Setter;
import org.pf4j.update.PluginInfo;

import java.nio.file.Path;

@Getter
@Setter
@AllArgsConstructor
@Builder
public class PluginInfoPropertiesDTO {

    private final StringProperty idProperty;
    private final StringProperty nameProperty;
    private final StringProperty descriptionProperty;
    private final StringProperty providerProperty;
    private final StringProperty projectUrlProperty;
    private final StringProperty repositoryIdProperty;
    private final StringProperty installedVersionProperty;
    private final StringProperty installableVersionProperty;
    private final StringProperty licenseProperty;
    private final ObjectProperty<Path> pathProperty;
    private final BooleanProperty disabledProperty;
    private final BooleanProperty bundledProperty;

    private final BooleanProperty upgradeableProperty;

    public static Callback<PluginInfoPropertiesDTO, Observable[]> extractor() {
        return (PluginInfoPropertiesDTO c) -> new Observable[]{
                c.idProperty,
                c.nameProperty,
                c.descriptionProperty,
                c.providerProperty,
                c.projectUrlProperty,
                c.repositoryIdProperty,
                c.installedVersionProperty,
                c.installableVersionProperty,
                c.licenseProperty,
                c.pathProperty,
                c.disabledProperty,
                c.bundledProperty
        };
    }

    public String getId() {
        return idProperty.getValue();
    }

    public String getName() {
        return nameProperty.getValue();
    }

    public String getProjectUrl() {
        return projectUrlProperty.getValue();
    }

    public String getDescription() {
        return descriptionProperty.getValue();
    }

    public String getProvider() {
        return providerProperty.getValue();
    }

    public String getLicense() {
        return licenseProperty.getValue();
    }

    public String getRepositoryId() {
        return repositoryIdProperty.getValue();
    }

    public String getInstalledVersion() {
        return installedVersionProperty.getValue();
    }

    public String getInstallableVersion() {
        return installableVersionProperty.getValue();
    }

    public Boolean getUpgradeable(){
        return upgradeableProperty.getValue();
    }

    public Path getPath() {
        return pathProperty.getValue();
    }

    public Boolean getDisabled(){
        return disabledProperty.getValue();
    }

    public Boolean getBundled() {
        return bundledProperty.getValue();
    }

    @Override
    public String toString() {
        return idProperty.getName();
    }

    @SuppressWarnings({"FieldCanBeLocal", "unused"})
    public static class PluginInfoPropertiesDTOBuilder {

        private StringProperty idProperty = new SimpleStringProperty();
        private StringProperty nameProperty = new SimpleStringProperty();
        private StringProperty descriptionProperty = new SimpleStringProperty();
        private StringProperty providerProperty = new SimpleStringProperty();
        private StringProperty projectUrlProperty = new SimpleStringProperty();
        private StringProperty repositoryIdProperty = new SimpleStringProperty();
        private StringProperty installedVersionProperty = new SimpleStringProperty();
        private StringProperty installableVersionProperty = new SimpleStringProperty();
        private StringProperty licenseProperty = new SimpleStringProperty();
        private ObjectProperty<Path> pathProperty = new SimpleObjectProperty<>();
        private BooleanProperty disabledProperty = new SimpleBooleanProperty();
        private BooleanProperty bundledProperty = new SimpleBooleanProperty();
        private BooleanProperty upgradeableProperty = new SimpleBooleanProperty();

        public PluginInfoPropertiesDTOBuilder id(String id) {
            this.idProperty.set(id);
            return this;
        }

        public PluginInfoPropertiesDTOBuilder name(String name) {
            this.nameProperty.set(name);
            return this;
        }

        public PluginInfoPropertiesDTOBuilder projectUrl(String projectUrl) {
            this.projectUrlProperty.set(projectUrl);
            return this;
        }

        public PluginInfoPropertiesDTOBuilder description(String description) {
            this.descriptionProperty.set(description);
            return this;
        }

        public PluginInfoPropertiesDTOBuilder provider(String provider) {
            this.providerProperty.set(provider);
            return this;
        }

        public PluginInfoPropertiesDTOBuilder license(String license) {
            this.licenseProperty.set(license);
            return this;
        }

        public PluginInfoPropertiesDTOBuilder installedVersion(String installedVersion) {
            this.installedVersionProperty.set(installedVersion);
            return this;
        }

        public PluginInfoPropertiesDTOBuilder installableVersion(String installableVersion) {
            this.installableVersionProperty.set(installableVersion);
            return this;
        }

        public PluginInfoPropertiesDTOBuilder repositoryId(String repositoryId) {
            this.repositoryIdProperty.set(repositoryId);
            return this;
        }

        public PluginInfoPropertiesDTOBuilder path(Path path){
            this.pathProperty.set(path);
            return this;
        }

        public PluginInfoPropertiesDTOBuilder disabled(boolean disabled){
            this.disabledProperty.set(disabled);
            return this;
        }

        public PluginInfoPropertiesDTOBuilder bundled(boolean bundled){
            this.bundledProperty.set(bundled);
            return this;
        }


        public PluginInfoPropertiesDTOBuilder upgradeable(boolean upgradeable){
            this.upgradeableProperty.set(upgradeable);
            return this;
        }

        public PluginInfoPropertiesDTO build() {
            return new PluginInfoPropertiesDTO(idProperty,
                    nameProperty,
                    descriptionProperty,
                    providerProperty,
                    projectUrlProperty,
                    repositoryIdProperty,
                    installedVersionProperty,
                    installableVersionProperty,
                    licenseProperty,
                    pathProperty,
                    disabledProperty,
                    bundledProperty,
                    upgradeableProperty
            );
        }
    }
}
