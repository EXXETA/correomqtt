package org.correomqtt.gui.controller;

import javafx.event.EventHandler;
import javafx.fxml.FXMLLoader;
import javafx.scene.Parent;
import javafx.scene.Scene;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.Pane;
import javafx.stage.Modality;
import javafx.stage.Stage;
import javafx.stage.StageStyle;
import javafx.stage.WindowEvent;
import org.correomqtt.business.provider.SettingsProvider;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.lang.reflect.InvocationTargetException;
import java.text.MessageFormat;
import java.util.Map;
import java.util.ResourceBundle;

abstract class BaseController {

    private static final Logger LOGGER = LoggerFactory.getLogger(BaseController.class);
    private static final ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale());

    static <C extends BaseController> LoaderResult<C> load(Class<C> controllerClazz, String fxml) {
        return load(controllerClazz,
                    fxml,
                    () -> controllerClazz.getDeclaredConstructor().newInstance());
    }

    static <C extends BaseController> LoaderResult<C> load(final Class<C> controllerClazz,
                                                                               final String fxml,
                                                                               final ConstructorMethod<C> constructorMethod) {

        FXMLLoader loader = new FXMLLoader(controllerClazz.getResource(fxml),
                ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale()));

        loader.setControllerFactory(param -> {
            try {
                return constructorMethod.construct();
            } catch (InstantiationException | InvocationTargetException | NoSuchMethodException | IllegalAccessException e) {
                throw new IllegalStateException(MessageFormat.format("Exception loading {0} from {1}: ", controllerClazz.getSimpleName(), fxml), e);
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
        showAsDialog(result, title, windowProperties,resizable, alwaysOnTop, closeHandler, keyHandler, 300, 400);

    }

    static <Z extends BaseController> void showAsDialog(LoaderResult<Z> result,
                                                        String title,
                                                        Map<Object, Object> windowProperties,
                                                        boolean resizable,
                                                        boolean alwaysOnTop,
                                                        final EventHandler<WindowEvent> closeHandler,
                                                        final EventHandler<KeyEvent> keyHandler,
                                                        int minWidth,
                                                        int minHeight) {

        Scene scene = new Scene(result.getMainPane());
        String cssPath = SettingsProvider.getInstance().getCssPath();
        if (cssPath != null) {
            scene.getStylesheets().add(cssPath);
        }
        Stage stage = new Stage();
        stage.initStyle(StageStyle.UTILITY);
        stage.setTitle(title);
        stage.setScene(scene);
        stage.setResizable(resizable);
        stage.setMinWidth(minWidth);
        stage.setMinHeight(minHeight);
        stage.setAlwaysOnTop(alwaysOnTop);
        stage.initModality(Modality.WINDOW_MODAL);
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
