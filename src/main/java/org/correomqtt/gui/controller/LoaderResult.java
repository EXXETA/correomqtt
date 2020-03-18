package org.correomqtt.gui.controller;

import javafx.scene.layout.Pane;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;

import java.util.ResourceBundle;

@Getter
@AllArgsConstructor
@Builder
class LoaderResult<C extends BaseController> {
    private final C controller;
    private final Pane mainPane;
    private final ResourceBundle resourceBundle;
}
