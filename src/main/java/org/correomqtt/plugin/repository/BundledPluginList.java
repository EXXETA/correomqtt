package org.correomqtt.plugin.repository;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.Setter;

import java.io.Serializable;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

@Getter
@Setter
@NoArgsConstructor
@AllArgsConstructor
@Builder
public class BundledPluginList {

    @Builder.Default
    private Map<String, BundledPlugins> versions = new HashMap<>();

    @Getter
    @Setter
    @NoArgsConstructor
    @AllArgsConstructor
    @Builder
    public static class BundledPlugins implements Serializable {

        @Builder.Default
        private List<String> install = new ArrayList<>();

        @Builder.Default
        private List<String> uninstall = new ArrayList<>();
    }

}
