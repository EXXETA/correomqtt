package org.correomqtt.core.utils;

import org.xml.sax.ErrorHandler;
import org.xml.sax.SAXException;
import org.xml.sax.SAXParseException;

public final class SilentSaxErrorHandler implements ErrorHandler {

    public static final SilentSaxErrorHandler INSTANCE = new SilentSaxErrorHandler();

    private SilentSaxErrorHandler() {
    }

    @Override
    public void warning(SAXParseException exception) {
    }

    @Override
    public void error(SAXParseException exception) throws SAXException {
        throw exception;
    }

    @Override
    public void fatalError(SAXParseException exception) throws SAXException {
        throw exception;
    }
}
