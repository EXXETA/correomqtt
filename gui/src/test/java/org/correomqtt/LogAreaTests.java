package org.correomqtt;

import org.junit.jupiter.api.Disabled;
import org.junit.jupiter.api.Test;
import org.testfx.matcher.base.NodeMatchers;

import static org.testfx.api.FxAssert.verifyThat;

@Disabled
class LogAreaTests extends TestBaseClass{

    @Test
    void logAreaVisibleTest(){
        verifyThat("#logTextArea",NodeMatchers.isVisible());
    }
}
