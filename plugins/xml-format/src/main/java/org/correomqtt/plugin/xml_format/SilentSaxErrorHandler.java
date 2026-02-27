package org.correomqtt.plugin.xml_format;

import org.xml.sax.ErrorHandler;
import org.xml.sax.SAXException;
import org.xml.sax.SAXParseException;

final class SilentSaxErrorHandler implements ErrorHandler {

    @Override
    public void warning(SAXParseException exception) {
        // Intentionally empty: warnings are non-fatal and safely ignored during XML validation
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
