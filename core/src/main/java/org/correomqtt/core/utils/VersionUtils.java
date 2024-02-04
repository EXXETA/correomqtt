package org.correomqtt.core.utils;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.apache.commons.io.IOUtils;
import org.apache.maven.artifact.versioning.ComparableVersion;
import org.correomqtt.core.exception.CorreoMqttUnableToCheckVersionException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.FileNotFoundException;
import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.net.HttpURLConnection;
import java.net.SocketTimeoutException;
import java.net.URL;
import java.net.UnknownHostException;
import java.nio.charset.StandardCharsets;
import java.util.Objects;

public class VersionUtils {

    private static final Logger LOGGER = LoggerFactory.getLogger(VersionUtils.class);

    private static String version;

    private VersionUtils() {
        // private constructor
    }

    public static String getVersion() {
        if (version == null) {
            try {
                version = IOUtils.toString(Objects.requireNonNull(VersionUtils.class.getResourceAsStream("version.txt")), StandardCharsets.UTF_8);
            } catch (IOException e) {
                LOGGER.error("Error reading version: ", e);
                version = "N/A";
            }
        }

        return version.trim();
    }

    /**
     * Check if a new version is available.
     *
     * @return The name of the tag on github if a new version exists, null otherwise.
     */
    public static String isNewerVersionAvailable() throws IOException, CorreoMqttUnableToCheckVersionException {

        URL url = new URL(VendorConstants.GITHUB_API_LATEST());
        HttpURLConnection connection = (HttpURLConnection) url.openConnection();
        connection.setRequestMethod("GET");
        connection.setRequestProperty("Content-Type", "application/json");
        connection.setRequestProperty("charset", "utf-8");
        connection.setConnectTimeout(5000);
        connection.setReadTimeout(5000);

        try {
            connection.connect();
            InputStream inputStream = connection.getInputStream();
            GithubApiLatestDTO latestDTO = new ObjectMapper().readValue(
                    new InputStreamReader(inputStream, StandardCharsets.UTF_8),
                    GithubApiLatestDTO.class);
            String tagName = latestDTO.getTagName();
            ComparableVersion latestGithubVersion = new ComparableVersion(tagName.replaceAll("[^0-9.]",""));
            ComparableVersion currentLocalVersion = new ComparableVersion(getVersion());

            if (latestGithubVersion.compareTo(currentLocalVersion) > 0) {
                LOGGER.info("There is a new release available on github! {}", VendorConstants.GITHUB_LATEST());
                return tagName;
            } else {
                LOGGER.info("Version is up to date or newer! {}", VendorConstants.GITHUB_LATEST());
                return null;
            }
        } catch (FileNotFoundException fnfe){
            LOGGER.warn("Unable to find {} while checking for new version. Plugin updates will also be skipped.", VendorConstants.GITHUB_API_LATEST());
        } catch (SocketTimeoutException ste){
            LOGGER.warn("Timeout checking for new version. Plugin updates will also be skipped.");
        } catch (UnknownHostException uhe) {
            LOGGER.warn("No internet connection for checking latest version. Plugin updates will also be skipped.");
        }
        throw new CorreoMqttUnableToCheckVersionException();
    }
}
