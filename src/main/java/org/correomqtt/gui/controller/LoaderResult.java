package org.correomqtt.gui.controller;

import javafx.scene.layout.Region;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;

import java.util.ResourceBundle;

@Getter
@AllArgsConstructor
@Builder
public class LoaderResult<C extends BaseController> {
    private final C controller;
    private final Region mainRegion;
    private final ResourceBundle resourceBundle;
}
