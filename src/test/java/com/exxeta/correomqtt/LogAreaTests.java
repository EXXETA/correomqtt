package com.exxeta.correomqtt;

import org.junit.Test;
import org.testfx.matcher.base.NodeMatchers;

import static org.testfx.api.FxAssert.verifyThat;

public class LogAreaTests extends TestBaseClass{

    @Test
    public void logAreaVisibleTest(){
        verifyThat("#logTextArea",NodeMatchers.isVisible());
    }
}
