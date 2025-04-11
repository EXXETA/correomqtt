package org.correomqtt.preloader;

import javafx.application.Preloader;
import lombok.AllArgsConstructor;
import lombok.Getter;

@AllArgsConstructor
@Getter
public class PreloaderNotification implements Preloader.PreloaderNotification {
    private String msg;

}
