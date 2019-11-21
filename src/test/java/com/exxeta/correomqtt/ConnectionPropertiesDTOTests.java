package com.exxeta.correomqtt;

import com.exxeta.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNotNull;

public class ConnectionPropertiesDTOTests {

    @Test
    public void testBuilder() {
        ConnectionPropertiesDTO props = ConnectionPropertiesDTO.builder()
                                                               .name("test")
                                                               .build();

        assertEquals(props.getName(), "test");
        assertNotNull(props.getUsernameProperty());
    }
}
