package org.correomqtt.gui.menuitem;

import javafx.scene.control.CustomMenuItem;
import javafx.scene.control.Label;
import lombok.Getter;
import org.correomqtt.plugin.manager.DetailViewManipulatorTask;

public class DetailViewManipulatorTaskMenuItem extends CustomMenuItem {

    @Getter
    private final DetailViewManipulatorTask task;

    public DetailViewManipulatorTaskMenuItem(DetailViewManipulatorTask task) {
        super(new Label(task.getName()));
        this.task = task;
    }
}
