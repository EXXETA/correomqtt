package com.exxeta.correomqtt;

import javafx.stage.Stage;
import org.junit.Before;
import org.testfx.framework.junit.ApplicationTest;

abstract class TestBaseClass extends ApplicationTest {

    @Before
    public void setUpClass() throws Exception {
        ApplicationTest.launch(CorreoMqtt.class);
    }

    @Override
    public void start(Stage stage){
        stage.show();
    }
}