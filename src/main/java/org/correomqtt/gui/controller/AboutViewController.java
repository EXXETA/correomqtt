package org.correomqtt.gui.controller;

import com.exxeta.correomqtt.business.utils.VersionUtils;
import com.exxeta.correomqtt.gui.model.WindowProperty;
import com.exxeta.correomqtt.gui.model.WindowType;
import com.exxeta.correomqtt.gui.utils.HostServicesHolder;
import com.exxeta.correomqtt.gui.utils.WindowHelper;
import javafx.event.ActionEvent;
import javafx.event.EventHandler;
import javafx.fxml.FXML;
import javafx.scene.control.Hyperlink;
import javafx.scene.control.Label;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.ResourceBundle;

public class AboutViewController extends BaseController {

    List<Hyperlink> links = new ArrayList<>();
    @FXML
    private Hyperlink hiveMqttClientRepo;
    @FXML
    private Hyperlink hiveMqttClientLicense;
    @FXML
    private Hyperlink JSONSimpleRepo;
    @FXML
    private Hyperlink JSONSimpleLicense;
    @FXML
    private Hyperlink jacksonDatabindRepo;
    @FXML
    private Hyperlink jacksonDatabindLicense;
    @FXML
    private Hyperlink gsonRepo;
    @FXML
    private Hyperlink gsonLicense;
    @FXML
    private Hyperlink richTextFXRepo;
    @FXML
    private Hyperlink richTextFXLicense;
    @FXML
    private Hyperlink flowlessRepo;
    @FXML
    private Hyperlink flowlessLicense;
    @FXML
    private Hyperlink lombokRepo;
    @FXML
    private Hyperlink lombokLicense;
    @FXML
    private Hyperlink langRepo;
    @FXML
    private Hyperlink langLicense;
    @FXML
    private Hyperlink ioRepo;
    @FXML
    private Hyperlink ioLicense;
    @FXML
    private Hyperlink logbackRepo;
    @FXML
    private Hyperlink logbackLicense;
    @FXML
    private Hyperlink sshjRepo;
    @FXML
    private Hyperlink sshjLicense;
    @FXML
    private Hyperlink fontAwesomeRepo;
    @FXML
    private Hyperlink fontAwesomeLicense;
    @FXML
    private Hyperlink exxetaLink;

    private static ResourceBundle resources;

    @FXML
    private Label appNameLabel;

    public static LoaderResult<AboutViewController> load() {
        return load(AboutViewController.class, "aboutView.fxml");
    }

    public static void showAsDialog() {

        Map<Object, Object> properties = new HashMap<>();
        properties.put(WindowProperty.WINDOW_TYPE, WindowType.ABOUT);

        if(WindowHelper.focusWindowIfAlreadyThere(properties)){
            return;
        }
        LoaderResult<AboutViewController> result = load();
        resources = result.getResourceBundle();
        showAsDialog(result, resources.getString("aboutViewControllerTitle"), properties, false, false, null, null);
    }


    @FXML
    public void initialize() {

        appNameLabel.setText("CorreoMQTT v" + VersionUtils.getVersion());

        hiveMqttClientRepo.getProperties().put("link", "https://github.com/hivemq/hivemq-mqtt-client/tree/v1.1.1");
        hiveMqttClientLicense.getProperties().put("link", "https://github.com/hivemq/hivemq-mqtt-client/blob/v1.1.1/LICENSE");
        JSONSimpleRepo.getProperties().put("link", "https://github.com/fangyidong/json-simple/tree/tag_release_1_1_1");
        JSONSimpleLicense.getProperties().put("link", "https://github.com/fangyidong/json-simple/blob/tag_release_1_1_1/LICENSE.txt");
        jacksonDatabindRepo.getProperties().put("link", "https://github.com/FasterXML/jackson-databind/tree/jackson-databind-2.9.7");
        jacksonDatabindLicense.getProperties().put("link", "https://github.com/FasterXML/jackson-databind/blob/jackson-databind-2.9.7/src/main/resources/META-INF/LICENSE");
        gsonRepo.getProperties().put("link", "https://github.com/google/gson/tree/gson-parent-2.8.5");
        gsonLicense.getProperties().put("link", "https://github.com/google/gson/blob/gson-parent-2.8.5/LICENSE");
        richTextFXRepo.getProperties().put("link", "https://github.com/FXMisc/RichTextFX/tree/v0.9.3");
        richTextFXLicense.getProperties().put("link", "https://github.com/FXMisc/RichTextFX/blob/v0.9.3/LICENSE");
        flowlessRepo.getProperties().put("link", "https://github.com/FXMisc/Flowless/tree/v0.6.1");
        flowlessLicense.getProperties().put("link", "https://github.com/FXMisc/Flowless/blob/v0.6.1/LICENSE");
        lombokRepo.getProperties().put("link", "https://projectlombok.org/");
        lombokLicense.getProperties().put("link", "https://projectlombok.org/LICENSE");
        langRepo.getProperties().put("link", "https://github.com/apache/commons-lang/tree/LANG_3_8");
        langLicense.getProperties().put("link", "https://github.com/apache/commons-lang/blob/LANG_3_8/NOTICE.txt");
        ioRepo.getProperties().put("link", "https://github.com/apache/commons-io");
        ioLicense.getProperties().put("link", "https://github.com/apache/commons-io/blob/master/NOTICE.txt");
        logbackRepo.getProperties().put("link", "https://github.com/qos-ch/logback/tree/v_1.2.3");
        logbackLicense.getProperties().put("link", "https://github.com/qos-ch/logback/blob/v_1.2.3/LICENSE.txt");
        sshjRepo.getProperties().put("link", "https://github.com/hierynomus/sshj/tree/v0.27.0");
        sshjLicense.getProperties().put("link", "https://github.com/hierynomus/sshj/blob/v0.27.0/LICENSE_HEADER");
        fontAwesomeRepo.getProperties().put("link", "https://github.com/FortAwesome/Font-Awesome");
        fontAwesomeLicense.getProperties().put("link", "https://fontawesome.com/license/free");
        exxetaLink.getProperties().put("link", "https://www.exxeta.com");

        links.addAll(Arrays.asList(
                hiveMqttClientRepo, hiveMqttClientLicense,
                JSONSimpleRepo, JSONSimpleLicense,
                jacksonDatabindRepo, jacksonDatabindLicense,
                gsonRepo, gsonLicense,
                richTextFXRepo, richTextFXLicense,
                flowlessRepo, flowlessLicense,
                lombokRepo, lombokLicense,
                langRepo, langLicense,
                ioRepo, ioLicense,
                logbackRepo, logbackLicense,
                sshjRepo, sshjLicense,
                fontAwesomeRepo, fontAwesomeLicense,
                exxetaLink)
        );

        for ( final Hyperlink hyperlink : links ) {
            hyperlink.setOnAction(new EventHandler<ActionEvent>() {
                @Override
                public void handle(ActionEvent t) {
                    HostServicesHolder.getInstance().getHostServices().showDocument(hyperlink.getProperties().get("link").toString());
                }
            });
        }
    }
}
