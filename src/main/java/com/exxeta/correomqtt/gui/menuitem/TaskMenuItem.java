package com.exxeta.correomqtt.gui.menuitem;

import com.exxeta.correomqtt.plugin.manager.PluginProtocolTask;
import javafx.scene.control.CustomMenuItem;
import javafx.scene.control.Label;
import lombok.Getter;

public class TaskMenuItem<T> extends CustomMenuItem {

    @Getter
    private final PluginProtocolTask<T> task;

    public TaskMenuItem(PluginProtocolTask<T> task) {
        super(new Label(task.getId()));
        this.task = task;
    }
}
