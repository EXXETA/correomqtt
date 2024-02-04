package org.correomqtt.gui.views.about;

import javafx.application.HostServices;
import javafx.scene.control.Hyperlink;
import javafx.scene.control.Label;
import javafx.scene.layout.HBox;

public class LicenseControl extends HBox {

    private final HostServices hostServices;

    public LicenseControl(License license,
                          HostServices hostServices) {
        super();
        this.hostServices = hostServices;
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
        control.setOnAction(t -> hostServices.showDocument(link));
    }
}

