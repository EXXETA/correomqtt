package org.correomqtt.plugin.base64;

import java.util.Base64;

public class Base64Utils {

    public static byte[] decode(byte[] input) {
        try {
            return Base64.getDecoder().decode(input);
        } catch (IllegalArgumentException e) {
            return input;
        }
    }

    public static byte[] encode(byte[] input) {
        return Base64.getEncoder().encode(input);
    }
}
