package org.correomqtt.gui.controller;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonElement;
import com.google.gson.JsonParser;
import org.w3c.dom.Document;
import org.xml.sax.InputSource;

import javax.xml.parsers.DocumentBuilder;
import javax.xml.parsers.DocumentBuilderFactory;
import javax.xml.transform.OutputKeys;
import javax.xml.transform.Transformer;
import javax.xml.transform.TransformerFactory;
import javax.xml.transform.dom.DOMSource;
import javax.xml.transform.stream.StreamResult;
import java.io.StringReader;
import java.io.StringWriter;

public abstract class BaseDetailSaveController extends BaseConnectionController {

    BaseDetailSaveController(String connectionId) {
        super(connectionId);
    }

    public String setGson(String finalMessage) {
        Gson gson = new GsonBuilder().setPrettyPrinting().create();
        JsonParser jp = new JsonParser();
        JsonElement je = jp.parse(finalMessage);
        String prettyJsonString = gson.toJson(je);
        finalMessage = prettyJsonString;

        return finalMessage;
    }

    public String setXml(String finalMessage) {
        try {
            DocumentBuilderFactory factory = DocumentBuilderFactory.newInstance();
            DocumentBuilder builder;
            builder = factory.newDocumentBuilder();
            Document document = builder.parse(new InputSource(new StringReader(finalMessage)));

            TransformerFactory tf = TransformerFactory.newInstance();
            Transformer transformer = tf.newTransformer();
            transformer.setOutputProperty(OutputKeys.OMIT_XML_DECLARATION, "yes");
            StringWriter writer = new StringWriter();
            transformer.transform(new DOMSource(document), new StreamResult(writer));
            finalMessage = writer.getBuffer().toString().replaceAll("(?m)^\\s*$[\n\r]{1,}", "");
        } catch (Exception ex){

        }

        return finalMessage;
    }
}
