package org.correomqtt.gui.controller;

import org.correomqtt.business.services.ConfigService;
import javafx.event.EventHandler;
import javafx.fxml.FXMLLoader;
import javafx.scene.Parent;
import javafx.scene.Scene;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.Pane;
import javafx.stage.Stage;
import javafx.stage.WindowEvent;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.lang.reflect.InvocationTargetException;
import java.util.Map;
import java.util.ResourceBundle;

abstract class BaseController {

    private static final Logger LOGGER = LoggerFactory.getLogger(BaseController.class);
    private static ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", ConfigService.getInstance().getSettings().getCurrentLocale());

    static <C extends BaseController, Z extends Class<C>> LoaderResult<C> load(Z controllerClazz, String fxml) {
        return load(controllerClazz,
                    fxml,
                    () -> controllerClazz.getDeclaredConstructor().newInstance());
    }

    static <C extends BaseController, Z extends Class<C>> LoaderResult<C> load(final Z controllerClazz,
                                                                               final String fxml,
                                                                               final ConstructorMethod<C> constructorMethod) {

        FXMLLoader loader = new FXMLLoader(controllerClazz.getResource(fxml),
                ResourceBundle.getBundle("org.correomqtt.i18n", ConfigService.getInstance().getSettings().getCurrentLocale()));

        loader.setControllerFactory(param -> {
            try {
                return constructorMethod.construct();
            } catch (InstantiationException | InvocationTargetException | NoSuchMethodException | IllegalAccessException e) {
                LOGGER.error("Exception loading {} from {}: ", controllerClazz.getSimpleName(), fxml, e);
                throw new IllegalStateException(e);
            }
        });

        Parent parent;
        try {
            parent = loader.load();
        } catch (IOException e) {
            LOGGER.error("Exception loading {} from {}", controllerClazz.getSimpleName(), fxml);
            throw new IllegalStateException(e);
        }

        return LoaderResult.<C>builder()
                .controller(loader.getController())
                .mainPane((Pane) parent)
                .resourceBundle(resources)
                .build();
    }

    static <Z extends BaseController> void showAsDialog(LoaderResult<Z> result,
                                                        String title,
                                                        Map<Object, Object> windowProperties,
                                                        boolean resizable,
                                                        boolean alwaysOnTop,
                                                        final EventHandler<WindowEvent> closeHandler,
                                                        final EventHandler<KeyEvent> keyHandler) {

        Scene scene = new Scene(result.getMainPane());
        String cssPath = ConfigService.getInstance().getCssPath();
        if (cssPath != null) {
            scene.getStylesheets().add(cssPath);
        }
        Stage stage = new Stage();
        stage.setTitle(title);
        stage.setScene(scene);
        stage.setResizable(resizable);
        stage.setAlwaysOnTop(alwaysOnTop);
        stage.show();
        if (closeHandler != null) {
            stage.getScene().getWindow().addEventFilter(WindowEvent.WINDOW_CLOSE_REQUEST, closeHandler);
        }
        if (keyHandler != null) {
            stage.getScene().getWindow().addEventFilter(KeyEvent.KEY_PRESSED, keyHandler);
        }
        stage.getScene().getWindow().getProperties().putAll(windowProperties);
    }

    @FunctionalInterface
    public interface ConstructorMethod<Z> {
        Z construct() throws InstantiationException, IllegalAccessException, InvocationTargetException, NoSuchMethodException;
    }
}
