package com.exxeta.correomqtt.business.utils;

import javafx.util.Pair;
import org.apache.commons.io.IOUtils;
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
import java.nio.charset.StandardCharsets;

public class VersionUtils {

    private static final Logger LOGGER = LoggerFactory.getLogger(VersionUtils.class);

    private static String version;

    private VersionUtils() {
        // private constructor
    }

    public static String getVersion() {
        if (version == null) {
            try {
                version = IOUtils.toString(VersionUtils.class.getResourceAsStream("version.txt"), StandardCharsets.UTF_8);
            } catch (IOException e) {
                LOGGER.error("Error reading version: ", e);
                version = "N/A";
            }
        }

        return version;
    }

    public static Pair<Boolean, String> isNewVersionAvailable() throws IOException, ParseException {

        URL url = new URL("https://api.github.com/repos/exxeta/correomqtt/releases/latest");
        HttpURLConnection connection = (HttpURLConnection) url.openConnection();
        connection.setRequestMethod("GET");
        connection.setRequestProperty("Content-Type", "application/json");
        connection.setRequestProperty("charset", "utf-8");
        connection.connect();

        InputStream inputStream = connection.getInputStream();
        JSONParser jsonParser = new JSONParser();
        JSONObject jsonObject = (JSONObject)jsonParser.parse(
                new InputStreamReader(inputStream, "UTF-8"));

        if (!jsonObject.get("tag_name").toString().contains(getVersion())) {
            System.out.println("There is a new release available on github!");
            return new Pair(true, jsonObject.get("tag_name"));
        } else {
            System.out.println("Version is up to date!");
            return new Pair(false, null);
        }
    }
}
