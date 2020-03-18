package org.correomqtt.gui.menuitem;

import org.correomqtt.plugin.manager.Task;
import javafx.scene.control.CustomMenuItem;
import javafx.scene.control.Label;
import lombok.Getter;

public class TaskMenuItem<T> extends CustomMenuItem {

    @Getter
    private final Task<T> task;

    public TaskMenuItem(Task<T> task) {
        super(new Label(task.getId()));
        this.task = task;
    }
}
