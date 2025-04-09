package org.correomqtt.gui.views;

import javafx.scene.layout.Region;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import org.correomqtt.gui.views.base.BaseController;

import java.util.ResourceBundle;

@Getter
@AllArgsConstructor
@Builder
public class LoaderResult<C extends BaseController> {
    private final C controller;
    private final Region mainRegion;
    private final ResourceBundle resourceBundle;
}
