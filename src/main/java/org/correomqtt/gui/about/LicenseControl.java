package org.correomqtt.gui.about;

import javafx.scene.control.Hyperlink;
import javafx.scene.control.Label;
import javafx.scene.layout.HBox;
import org.correomqtt.gui.utils.HostServicesHolder;

public class LicenseControl extends HBox {

    public LicenseControl(License license) {
        super();
        Hyperlink repoLink = new Hyperlink();
        repoLink.setText(license.name());
        activateLink(repoLink, license.repoLink());
        Label label = new Label();
        label.setPrefHeight(23);
        label.setText("licensed under");
        Hyperlink licenseLink = new Hyperlink();
        licenseLink.setText(license.license());
        activateLink(licenseLink, license.licenseLink());
        this.getChildren().addAll(repoLink, label, licenseLink);
    }

    private void activateLink(Hyperlink control, String link) {
        control.setOnAction(t -> HostServicesHolder.getInstance()
                .getHostServices()
                .showDocument(link)
        );
    }
}

