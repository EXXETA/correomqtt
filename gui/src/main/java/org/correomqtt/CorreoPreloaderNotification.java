package org.correomqtt;

import javafx.application.Preloader;
import lombok.AllArgsConstructor;
import lombok.Getter;

@AllArgsConstructor
@Getter
public class CorreoPreloaderNotification implements Preloader.PreloaderNotification {
    private String msg;

}
