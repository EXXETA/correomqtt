package org.correomqtt;

import javafx.application.HostServices;
import lombok.Getter;
import lombok.Setter;
import org.correomqtt.di.SingletonBean;

@Getter
@Setter
@SingletonBean
public class HostServicesWrapper {
    private HostServices hostServices;
}
