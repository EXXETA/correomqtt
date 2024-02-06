package org.correomqtt.core;

import org.correomqtt.core.utils.DirectoryUtils;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.di.SoyEvents;

@DefaultBean
public class CorreoCore {

    private final SoyEvents soyEvents;

    @Inject
    public CorreoCore(SoyEvents soyEvents) {
        this.soyEvents = soyEvents;
    }

    public void init() {
        System.setProperty("correo.configDirectory", DirectoryUtils.getTargetDirectoryPath());
    }
}
