package org.correomqtt.core.plugin;

import com.github.zafarkhaja.semver.Version;
import org.pf4j.DefaultVersionManager;
import org.pf4j.util.StringUtils;

public class NoSemverVersionManager extends DefaultVersionManager {

    /**
     * Normalizes SNAPSHOT versions to release versions for semver compatibility.
     * Replaces "1.0-SNAPSHOT" with "1.0.0", etc.
     */
    private String normalizeVersion(String version) {
        if (version != null && version.endsWith("-SNAPSHOT")) {
            String baseVersion = version.substring(0, version.length() - "-SNAPSHOT".length());
            // If the base version doesn't have a patch number, add .0
            if (baseVersion.matches("\\d+\\.\\d+")) {
                return baseVersion + ".0";
            }
        }
        return version;
    }

    @Override
    public boolean checkVersionConstraint(String version, String constraint) {
        String normalizedVersion = normalizeVersion(version);
        return StringUtils.isNullOrEmpty(constraint) || "*".equals(constraint) || Version.valueOf(normalizedVersion).satisfies(constraint);
    }

    @Override
    public int compareVersions(String v1, String v2) {
        String normalizedV1 = normalizeVersion(v1);
        String normalizedV2 = normalizeVersion(v2);
        return Version.valueOf(normalizedV1).compareTo(Version.valueOf(normalizedV2));
    }
}