package org.correomqtt.business.utils;

import org.apache.commons.io.IOUtils;
import org.apache.maven.artifact.versioning.ComparableVersion;
import org.json.simple.JSONObject;
import org.json.simple.parser.JSONParser;
import org.json.simple.parser.ParseException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.net.HttpURLConnection;
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

        return version;
    }

    /**
     * Check if a new version is available.
     *
     * @return The name of the tag on github if a new version exists, null otherwise.
     */
    public static String isNewerVersionAvailable() throws IOException, ParseException {

        URL url = new URL(VendorConstants.GITHUB_API_LATEST());
        HttpURLConnection connection = (HttpURLConnection) url.openConnection();
        connection.setRequestMethod("GET");
        connection.setRequestProperty("Content-Type", "application/json");
        connection.setRequestProperty("charset", "utf-8");
        try {
            connection.connect();
            InputStream inputStream = connection.getInputStream();
            JSONParser jsonParser = new JSONParser();
            JSONObject jsonObject = (JSONObject)jsonParser.parse(
                    new InputStreamReader(inputStream, StandardCharsets.UTF_8));

            ComparableVersion latestGithubVersion = new ComparableVersion(jsonObject.get("tag_name").toString().replaceAll("[^0-9\\.]",""));
            ComparableVersion currentLocalVersion = new ComparableVersion(getVersion());

            if (latestGithubVersion.compareTo(currentLocalVersion) > 0) {
                LOGGER.info("There is a new release available on github!");
                return jsonObject.get("tag_name").toString();
            } else {
                LOGGER.info("Version is up to date or newer!");
                return null;
            }
        } catch (UnknownHostException uhe) {
            LOGGER.error("No internet connection for checking latest version");
            return null;
        }
    }
}
