package org.correomqtt;

import javafx.stage.Stage;
import org.junit.jupiter.api.BeforeEach;
import org.testfx.framework.junit5.ApplicationTest;

abstract class TestBaseClass extends ApplicationTest {

    @BeforeEach
    public void setUpClass() throws Exception {
        ApplicationTest.launch(FxApplication.class);
    }

    @Override
    public void start(Stage stage){
        stage.show();
    }
}