package org.correomqtt.plugin.xml_xsd_validator;

import dagger.Component;
import org.correomqtt.MainComponent;
import org.correomqtt.core.fileprovider.PluginConfigProvider;
import org.correomqtt.core.plugin.spi.MessageValidatorHook;
import org.correomqtt.gui.plugin.ExtensionComponent;
import org.pf4j.Extension;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.xml.sax.SAXException;

import org.correomqtt.core.cdi.Inject;
import javax.xml.XMLConstants;
import javax.xml.transform.stream.StreamSource;
import javax.xml.validation.Schema;
import javax.xml.validation.SchemaFactory;
import javax.xml.validation.Validator;
import java.io.File;
import java.io.IOException;
import java.io.StringReader;

@Extension
public class XmlXsdValidator implements MessageValidatorHook<XmlXsdValidatorConfig> {

    private static final Logger LOGGER = LoggerFactory.getLogger(XmlXsdValidator.class);
    private final PluginConfigProvider pluginConfigProvider;

    private String schemaFile;


    @Component(dependencies = MainComponent.class)
    public interface Factory extends ExtensionComponent<XmlXsdValidator> {

    }

    @Inject
    XmlXsdValidator(PluginConfigProvider pluginConfigProvider) {
        this.pluginConfigProvider = pluginConfigProvider;
    }


    @Override
    public void onConfigReceived(XmlXsdValidatorConfig config) {
        schemaFile = config.getSchema();
    }

    @Override
    public Validation isMessageValid(String payload) {
        try {
            SchemaFactory factory = SchemaFactory.newInstance(XMLConstants.W3C_XML_SCHEMA_NS_URI);
            factory.setProperty(XMLConstants.ACCESS_EXTERNAL_DTD, "");
            factory.setProperty(XMLConstants.ACCESS_EXTERNAL_SCHEMA, "");
            Schema schema = factory.newSchema(new File(pluginConfigProvider.getPluginPath(), schemaFile));
            Validator validator = schema.newValidator();
            validator.validate(new StreamSource(new StringReader(payload)));
        } catch (IOException | SAXException e) {
            LOGGER.error(e.getMessage());
            return new Validation(false, e.getMessage());
        } catch (SecurityException e) {
            LOGGER.error("Please put your schema file into {}", pluginConfigProvider.getPluginPath());
            return new Validation(false, "XSD file not found");
        }
        return new Validation(true, schemaFile);
    }
}
