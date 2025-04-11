# Advance message validator for the CorreoMQTT-Client

## Installation
1. Compile the source code using `mvn clean package`
2. Copy the `.jar` into the `plugins/jars` directory (open through the application)  

## Configuration
This plugin can combine other validators by using Boolean Algebra.

```xml
<MessageValidatorHook>
  <task id="com/+/example">
      <plugin name="advanced-validator-plugin">
        <plugin name="contains-string-validator-plugin" extensionId="ignoreCase">
          <string>test</string>
        </plugin>
        <plugin name="contains-string-validator-plugin" extensionId="ignoreCase">
          <string>another</string>
        </plugin>
        <plugin name="contains-string-validator-plugin" extensionId="caseSensitive">
          <string>okay</string>
        </plugin>
      </plugin>
  </task>
</MessageValidatorHook>
```

## Caveat
So far this is only a proof of concept to show that one plugin can load and make use of other plugins.
The structure to define the Boolean Algebra as well as parsing it is yet to be implemented. 
