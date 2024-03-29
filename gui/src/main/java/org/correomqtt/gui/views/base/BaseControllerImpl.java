package org.correomqtt.gui.views.base;

import javafx.event.EventHandler;
import javafx.fxml.FXMLLoader;
import javafx.scene.Parent;
import javafx.scene.Scene;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.Region;
import javafx.stage.Modality;
import javafx.stage.Stage;
import javafx.stage.StageStyle;
import javafx.stage.WindowEvent;
import org.correomqtt.core.CoreManager;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.views.LoaderResult;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.lang.reflect.InvocationTargetException;
import java.text.MessageFormat;
import java.util.Map;
import java.util.ResourceBundle;

public abstract class BaseControllerImpl implements BaseController {

    private static final Logger LOGGER = LoggerFactory.getLogger(BaseControllerImpl.class);
    protected final CoreManager coreManager;
    protected final ThemeManager themeManager;
    private final ResourceBundle resources;

    @FunctionalInterface
    public interface ConstructorMethod<Z> {
        Z construct() throws InstantiationException, IllegalAccessException, InvocationTargetException, NoSuchMethodException;
    }

    protected BaseControllerImpl(CoreManager coreManager,
                                 ThemeManager themeManager) {
        this.coreManager = coreManager;
        this.themeManager = themeManager;
        resources = ResourceBundle.getBundle("org.correomqtt.i18n", coreManager.getSettingsManager().getSettings().getCurrentLocale());
    }

    protected <C extends BaseControllerImpl> LoaderResult<C> load(Class<C> controllerClazz, String fxml) {
        return load(controllerClazz,
                fxml,
                () -> controllerClazz.getDeclaredConstructor().newInstance()); //TODO
    }

    protected <C extends BaseControllerImpl> LoaderResult<C> load(final Class<C> controllerClazz,
                                                                  final String fxml,
                                                                  final ConstructorMethod<C> constructorMethod) {

        FXMLLoader loader = new FXMLLoader(controllerClazz.getResource(fxml),
                ResourceBundle.getBundle("org.correomqtt.i18n", coreManager.getSettingsManager().getSettings().getCurrentLocale()));

        loader.setControllerFactory(param -> {
            try {
                return constructorMethod.construct();
            } catch (InstantiationException | InvocationTargetException | NoSuchMethodException |
                     IllegalAccessException e) {
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
                .mainRegion((Region) parent)
                .resourceBundle(resources)
                .build();
    }

    protected <Z extends BaseControllerImpl> void showAsDialog(LoaderResult<Z> result,
                                                               String title,
                                                               Map<Object, Object> windowProperties,
                                                               boolean resizable,
                                                               boolean alwaysOnTop,
                                                               final EventHandler<WindowEvent> closeHandler,
                                                               final EventHandler<KeyEvent> keyHandler) {
        showAsDialog(result, title, windowProperties, resizable, alwaysOnTop, closeHandler, keyHandler, 300, 400);

    }

    protected <Z extends BaseControllerImpl> void showAsDialog(LoaderResult<Z> result,
                                                               String title,
                                                               Map<Object, Object> windowProperties,
                                                               boolean resizable,
                                                               boolean alwaysOnTop,
                                                               final EventHandler<WindowEvent> closeHandler,
                                                               final EventHandler<KeyEvent> keyHandler,
                                                               int minWidth,
                                                               int minHeight) {

        Scene scene = new Scene(result.getMainRegion());
        scene.setFill(themeManager.getActiveTheme().getBackgroundColor());
        String cssPath = themeManager.getCssPath();
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
}
