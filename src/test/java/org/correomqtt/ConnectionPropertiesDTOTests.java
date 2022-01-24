package org.correomqtt;

import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;


public class ConnectionPropertiesDTOTests {

    @Test
    public void testBuilder() {
        ConnectionPropertiesDTO props = ConnectionPropertiesDTO.builder()
                                                               .name("test")
                                                               .build();

        assertEquals("test", props.getName());
        assertNotNull(props.getUsernameProperty());
    }
}
