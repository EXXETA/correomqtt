# Xml-Xsd-Validator plugin for CorreoMQTT

## Installation
1. Place the `XmlXsdValidatorPlugin.jar` into the `plugins/jars` directory  
2. Configure the validator in the `protocol.xml` (see example)
3. Put your `example.xsd` into `plugins/config/xml-xsd-validator-plugin/`
### protocol.xml
```xml
<?xml version="1.0" encoding="UTF-8"?>
<protocol>
  <tasks>
    ...
    <MessageValidatorHook>
      <task name="/test/+/example">
        <plugin name="xml-xsd-validator-plugin">
          <schema>example.xsd</schema>
        </plugin>
      </task>
      ...
    </MessageValidatorHook>
  </tasks>
  ...
</protocol>
```

### XML which conforms to the example.xsd
```xml
<?xml version="1.0" encoding="UTF-8"?>  
<note>
  <to>Tove</to>
  <from>Jani</from>
  <heading>Reminder</heading>
  <body>Don't forget me this weekend!</body>
</note>
```

### XML which doesn't conform to the example.xsd
```xml
<?xml version="1.0" encoding="UTF-8"?>  
<note>
  <to>Tove</to>
  <from>Jani</from>
</note>
```
