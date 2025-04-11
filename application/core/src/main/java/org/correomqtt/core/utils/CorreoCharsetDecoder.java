package org.correomqtt.core.utils;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.nio.ByteBuffer;
import java.nio.charset.CharacterCodingException;
import java.nio.charset.CharsetDecoder;
import java.nio.charset.CodingErrorAction;
import java.nio.charset.StandardCharsets;

public class CorreoCharsetDecoder {

    private static final Logger LOGGER = LoggerFactory.getLogger(CorreoCharsetDecoder.class);

    private CorreoCharsetDecoder() {
        // private Constructor
    }

    public static String decode(byte[] input) {

        // Currently only UTF-8 is supported. Later we may provide support for other charsets via UI.

        CharsetDecoder decoder = StandardCharsets.UTF_8.newDecoder();
        decoder.onMalformedInput(CodingErrorAction.REPLACE)
               .onUnmappableCharacter(CodingErrorAction.REPLACE)
               .replaceWith("?");

        try {
            return decoder.decode(ByteBuffer.wrap(input)).toString();
        } catch (CharacterCodingException e) {
            LOGGER.warn("Unable to decode input byte array to string.");
            return "?";
        }
    }
}
