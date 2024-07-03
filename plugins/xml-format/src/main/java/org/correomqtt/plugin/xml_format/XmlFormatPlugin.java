package org.correomqtt.plugin.xml_format;

import org.correomqtt.gui.plugin.spi.DetailViewFormatHook;
import org.fxmisc.richtext.model.StyleSpans;
import org.fxmisc.richtext.model.StyleSpansBuilder;
import org.pf4j.Extension;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.w3c.dom.Document;
import org.w3c.dom.Node;
import org.w3c.dom.NodeList;
import org.xml.sax.InputSource;
import org.xml.sax.SAXException;

import javax.xml.XMLConstants;
import javax.xml.parsers.DocumentBuilder;
import javax.xml.parsers.DocumentBuilderFactory;
import javax.xml.parsers.ParserConfigurationException;
import javax.xml.transform.OutputKeys;
import javax.xml.transform.Transformer;
import javax.xml.transform.TransformerConfigurationException;
import javax.xml.transform.TransformerException;
import javax.xml.transform.TransformerFactory;
import javax.xml.transform.dom.DOMSource;
import javax.xml.transform.stream.StreamResult;
import javax.xml.xpath.XPathConstants;
import javax.xml.xpath.XPathExpression;
import javax.xml.xpath.XPathExpressionException;
import javax.xml.xpath.XPathFactory;
import java.io.IOException;
import java.io.StringReader;
import java.io.StringWriter;
import java.util.Collection;
import java.util.Collections;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@Extension
public class XmlFormatPlugin implements DetailViewFormatHook {

    private static final Logger LOGGER = LoggerFactory.getLogger(XmlFormatPlugin.class);

    private static final Pattern XML_TAG = Pattern.compile("(?<ELEMENT>(?<OPENBRACKET></?\\h*)(?<ELEMENTNAME>\\??[a-zA-Z][a-zA-Z0-9-_]*)(?<ATTRIBUTESECTION>[^<>]*)(?<CLOSEBRACKET>\\h*/?\\??>))"
            + "|(?<COMMENT><!--[^<>]+-->)");
    private static final Pattern ATTRIBUTES = Pattern.compile("(?<ATTRIBUTENAME>[a-zA-Z][a-zA-Z0-9-_]*\\h*)(?<EQUALSYMBOL>=)(?<ATTRIBUTEVALUE>\\h*\"[^\"]+\")");

    private static final String GROUP_OPEN_BRACKET = "OPENBRACKET";
    private static final String GROUP_ELEMENT_NAME = "ELEMENTNAME";
    private static final String GROUP_ATTRIBUTES_SECTION = "ATTRIBUTESECTION";
    private static final String GROUP_CLOSE_BRACKET = "CLOSEBRACKET";

    private static final String GROUP_ATTRIBUTE_NAME = "ATTRIBUTENAME";
    private static final String GROUP_EQUAL_SYMBOL = "EQUALSYMBOL";
    private static final String GROUP_ATTRIBUTE_VALUE = "ATTRIBUTEVALUE";

    private static final String COMMENT_CLASS = "commentXML";
    private static final String TAG_CLASS = "tagmarkXML";
    private static final String ANY_CLASS = "anytagXML";
    private static final String ATTRIBUTE_CLASS = "attributeXML";
    private static final String VALUE_CLASS = "avalueXML";
    private static final String EQUALSYMBOL_CLASS = TAG_CLASS;

    private String text;
    private Document xmlDocument;

    // https://stackoverflow.com/a/33564346
    private static void trimWhitespace(Node node) {
        NodeList children = node.getChildNodes();
        for (int i = 0; i < children.getLength(); ++i) {
            Node child = children.item(i);
            if (child.getNodeType() == Node.TEXT_NODE) {
                child.setTextContent(child.getTextContent().trim());
            }
            trimWhitespace(child);
        }
    }

    // https://github.com/FXMisc/RichTextFX/blob/5d64bd7ef211292ec096b5b152aa79ee934e4678/richtextfx-demos/src/main/java/org/fxmisc/richtext/demo/XMLEditorDemo.java
    @Override
    public StyleSpans<Collection<String>> getFxSpans() {

        String prettyText = getPrettyString();

        Matcher matcher = XML_TAG.matcher(prettyText);
        int lastKwEnd = 0;
        StyleSpansBuilder<Collection<String>> spansBuilder = new StyleSpansBuilder<>();
        while (matcher.find()) {
            spansBuilder.add(Collections.emptyList(), matcher.start() - lastKwEnd);
            if (matcher.group("COMMENT") != null) {
                spansBuilder.add(Collections.singleton(COMMENT_CLASS), matcher.end() - matcher.start());
            } else {
                if (matcher.group("ELEMENT") != null) {
                    String attributesText = matcher.group(GROUP_ATTRIBUTES_SECTION);

                    spansBuilder.add(Collections.singleton(TAG_CLASS), matcher.end(GROUP_OPEN_BRACKET) - matcher.start(GROUP_OPEN_BRACKET));
                    spansBuilder.add(Collections.singleton(ANY_CLASS), matcher.end(GROUP_ELEMENT_NAME) - matcher.end(GROUP_OPEN_BRACKET));

                    if (!attributesText.isEmpty()) {

                        lastKwEnd = 0;

                        Matcher amatcher = ATTRIBUTES.matcher(attributesText);
                        while (amatcher.find()) {
                            spansBuilder.add(Collections.emptyList(), amatcher.start() - lastKwEnd);
                            spansBuilder.add(Collections.singleton(ATTRIBUTE_CLASS), amatcher.end(GROUP_ATTRIBUTE_NAME) - amatcher.start(GROUP_ATTRIBUTE_NAME));
                            spansBuilder.add(Collections.singleton(EQUALSYMBOL_CLASS), amatcher.end(GROUP_EQUAL_SYMBOL) - amatcher.end(GROUP_ATTRIBUTE_NAME));
                            spansBuilder.add(Collections.singleton(VALUE_CLASS), amatcher.end(GROUP_ATTRIBUTE_VALUE) - amatcher.end(GROUP_EQUAL_SYMBOL));
                            lastKwEnd = amatcher.end();
                        }
                        if (attributesText.length() > lastKwEnd)
                            spansBuilder.add(Collections.emptyList(), attributesText.length() - lastKwEnd);
                    }

                    lastKwEnd = matcher.end(GROUP_ATTRIBUTES_SECTION);
                    spansBuilder.add(Collections.singleton(TAG_CLASS), matcher.end(GROUP_CLOSE_BRACKET) - lastKwEnd);
                }
            }
            lastKwEnd = matcher.end();
        }
        spansBuilder.add(Collections.emptyList(), prettyText.length() - lastKwEnd);
        return spansBuilder.create();
    }

    @Override
    public void setText(String text) {
        this.text = text;
        this.xmlDocument = createXmlDocument();
    }

    @Override
    public boolean isValid() {
        return getXmlDocument() != null;
    }

    private Document getXmlDocument() {
        if (xmlDocument == null) {
            xmlDocument = createXmlDocument();
        }

        return xmlDocument;
    }

    private Document createXmlDocument() {
        DocumentBuilderFactory factory = DocumentBuilderFactory.newInstance();
        try {
            factory.setFeature(XMLConstants.FEATURE_SECURE_PROCESSING, true);
        } catch (ParserConfigurationException e) {
            LOGGER.debug("Could not configure document builder factory. ", e);
            return null;
        }
        DocumentBuilder builder;
        try {
            builder = factory.newDocumentBuilder();
            xmlDocument = builder.parse(new InputSource(new StringReader(text)));
        } catch (ParserConfigurationException | SAXException | IOException e) {
            LOGGER.debug("Could parse message as xml: {}", e.getMessage());
            return null;
        }
        removeEmptyNodes();
        trimWhitespace(xmlDocument.getDocumentElement());
        return xmlDocument;
    }

    // https://stackoverflow.com/a/25866191
    private void removeEmptyNodes() {
        xmlDocument.getDocumentElement().normalize();
        XPathExpression xpath;
        NodeList blankTextNodes;
        try {
            xpath = XPathFactory.newInstance().newXPath().compile("//text()[normalize-space(.) = '']");
            blankTextNodes = (NodeList) xpath.evaluate(xmlDocument, XPathConstants.NODESET);
        } catch (XPathExpressionException e) {
            LOGGER.debug("Problem traversing xml document. ", e);
            return;
        }

        for (int i = 0; i < blankTextNodes.getLength(); i++) {
            blankTextNodes.item(i).getParentNode().removeChild(blankTextNodes.item(i));
        }
    }

    @Override
    public String getPrettyString() {
        TransformerFactory transformerFactory = TransformerFactory.newInstance();
        Transformer transformer;
        try {
            transformerFactory.setFeature(XMLConstants.FEATURE_SECURE_PROCESSING, true);
            transformer = transformerFactory.newTransformer();
        } catch (TransformerConfigurationException e) {
            LOGGER.debug("Problem configure xml transformer. ", e);
            return text;
        }
        transformer.setOutputProperty(OutputKeys.INDENT, "yes");
        transformer.setOutputProperty("{http://xml.apache.org/xslt}indent-amount", "2");
        transformer.setOutputProperty(OutputKeys.OMIT_XML_DECLARATION, "no");
        DOMSource source = new DOMSource(getXmlDocument());
        StringWriter strWriter = new StringWriter();
        StreamResult result = new StreamResult(strWriter);

        try {
            transformer.transform(source, result);
        } catch (TransformerException e) {
            LOGGER.debug("Problem transforming xml. ", e);
            return text;
        }

        return strWriter.getBuffer().toString();
    }
}
