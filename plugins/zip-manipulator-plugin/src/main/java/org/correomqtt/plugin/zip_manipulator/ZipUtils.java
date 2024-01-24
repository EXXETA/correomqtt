package org.correomqtt.plugin.zip_manipulator;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.util.zip.GZIPInputStream;
import java.util.zip.GZIPOutputStream;

public class ZipUtils {

    private static Logger LOGGER = LoggerFactory.getLogger(ZipUtils.class);

    public static byte[] zip(byte[] text) {
        if ((text == null) || (text.length == 0)) {
            throw new IllegalArgumentException("Cannot zip null or empty text");
        }

        try (ByteArrayOutputStream byteArrayOutputStream = new ByteArrayOutputStream()) {
            try (GZIPOutputStream gzipOutputStream = new GZIPOutputStream(byteArrayOutputStream)) {
                gzipOutputStream.write(text);
            }
            return byteArrayOutputStream.toByteArray();
        } catch (IOException e) {
            throw new RuntimeException("Failed to zip content", e);
        }
    }

    public static byte[] unzip(final byte[] compressed) {
        if (compressed.length == 0) {
            LOGGER.warn("Cannot unzip null or empty bytes");
            return compressed;
        }
        if (!isZipped(compressed)) {
            LOGGER.warn("Not a valid zip");
            return compressed;
        }

        try (ByteArrayInputStream byteArrayInputStream = new ByteArrayInputStream(compressed)) {
            try (GZIPInputStream gzipInputStream = new GZIPInputStream(byteArrayInputStream)) {
                return gzipInputStream.readAllBytes();
            }
        } catch (IOException e) {
            LOGGER.warn("Failed to unzip content", e);
            return compressed;
        }
    }

    private static boolean isZipped(final byte[] compressed) {
        return (compressed[0] == (byte) (GZIPInputStream.GZIP_MAGIC)) && (compressed[1] == (byte) (GZIPInputStream.GZIP_MAGIC >> 8));
    }
}
