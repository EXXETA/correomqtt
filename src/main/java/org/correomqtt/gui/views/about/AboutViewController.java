package org.correomqtt.gui.views.about;

import com.fasterxml.jackson.databind.ObjectMapper;
import javafx.fxml.FXML;
import javafx.scene.control.Hyperlink;
import javafx.scene.control.Label;
import javafx.scene.layout.HBox;
import javafx.scene.layout.VBox;
import org.correomqtt.business.utils.VersionUtils;
import org.correomqtt.gui.model.WindowProperty;
import org.correomqtt.gui.model.WindowType;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.utils.HostServicesHolder;
import org.correomqtt.gui.utils.WindowHelper;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;

import java.io.IOException;
import java.net.URL;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.ResourceBundle;

public class AboutViewController extends BaseControllerImpl {

    private static final URL LICENSE_JSON = AboutViewController.class.getResource("/META-INF/resources/licenses.json");

    @FXML
    private HBox libsHeadline;
    @FXML
    private VBox contentHolder;
    @FXML
    private HBox iconsHeadline;

    List<Hyperlink> links = new ArrayList<>();

    @FXML
    private Hyperlink exxetaLink;

    @FXML
    private Label appNameLabel;

    public static LoaderResult<AboutViewController> load() {
        return load(AboutViewController.class, "aboutView.fxml");
    }

    public static void showAsDialog() {

        Map<Object, Object> properties = new HashMap<>();
        properties.put(WindowProperty.WINDOW_TYPE, WindowType.ABOUT);

        if (WindowHelper.focusWindowIfAlreadyThere(properties)) {
            return;
        }
        LoaderResult<AboutViewController> result = load();
        ResourceBundle resources = result.getResourceBundle();
        showAsDialog(result, resources.getString("aboutViewControllerTitle"), properties, false, false, null, null);
    }


    @FXML
    private void initialize() {

        appNameLabel.setText("CorreoMQTT v" + VersionUtils.getVersion());

        Licenses licenses;
        try {
            licenses = new ObjectMapper().readValue(LICENSE_JSON, Licenses.class);
        } catch (IOException e) {
            AlertHelper.unexpectedAlert(e);
            throw new IllegalStateException(e);
        }

        addLicenses(licenses.libs(), libsHeadline);
        addLicenses(licenses.icons(), iconsHeadline);

        exxetaLink.getProperties().put("link", "https://www.exxeta.com");

        exxetaLink.setOnAction(t -> HostServicesHolder.getInstance()
                .getHostServices()
                .showDocument(exxetaLink.getProperties().get("link").toString())
        );

    }

    private void addLicenses(List<License> licenseList, HBox headline) {
        int index = contentHolder.getChildren().indexOf(headline);
        for (License license : licenseList) {
            index++;
            LicenseControl licenseControl = new LicenseControl(license);
            contentHolder.getChildren().add(index, licenseControl);
        }
    }
}
