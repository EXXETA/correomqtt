package org.correomqtt.plugins.systopic.controller;

import javafx.application.Platform;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.event.EventHandler;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.scene.Parent;
import javafx.scene.Scene;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.layout.Pane;
import javafx.stage.Stage;
import javafx.stage.WindowEvent;
import org.correomqtt.core.CoreManager;
import org.correomqtt.core.model.ConnectionConfigDTO;
import org.correomqtt.core.model.MessageDTO;
import org.correomqtt.core.model.Qos;
import org.correomqtt.core.pubsub.IncomingMessageEvent;
import org.correomqtt.core.pubsub.PubSubTaskFactories;
import org.correomqtt.core.pubsub.SubscribeEvent;
import org.correomqtt.core.pubsub.SubscribeFailedEvent;
import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.di.Observes;
import org.correomqtt.gui.model.SubscriptionPropertiesDTO;
import org.correomqtt.gui.model.WindowProperty;
import org.correomqtt.gui.model.WindowType;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.transformer.SubscriptionTransformer;
import org.correomqtt.gui.utils.ClipboardHelper;
import org.correomqtt.gui.utils.WindowHelper;
import org.correomqtt.plugins.systopic.model.SysTopic;
import org.correomqtt.plugins.systopic.model.SysTopicPropertiesDTO;
import org.correomqtt.plugins.systopic.model.SysTopicTransformer;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.time.LocalDateTime;
import java.time.ZoneOffset;
import java.util.Comparator;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Map;
import java.util.ResourceBundle;
import java.util.Set;
import java.util.stream.Collectors;

@DefaultBean
public class SysTopicViewController {

    private static final Logger LOGGER = LoggerFactory.getLogger(SysTopicViewController.class);
    private static final String SYS_TOPIC = "$SYS/#";
    private final ResourceBundle resources;
    private final SubscriptionPropertiesDTO subscriptionDTO = SubscriptionPropertiesDTO.builder()
            .topic(SYS_TOPIC)
            .qos(Qos.AT_LEAST_ONCE)
            .hidden(true)
            .build();
    private final CoreManager coreManager;
    private final PubSubTaskFactories pubSubTaskFactories;
    private final ThemeManager themeManager;
    private final SysTopicCellControllerFactory sysTopicCellControllerFactory;
    @FXML
    private Label connectionStatusLabel;
    @FXML
    private Label lastUpdateLabel;
    @FXML
    private ListView<SysTopicPropertiesDTO> listView;
    @FXML
    private Button copyToClipboardButton;

    private String connectionId;

    private ObservableList<SysTopicPropertiesDTO> sysTopics;

    private final Set<SysTopicPropertiesDTO> plainSysTopics = new HashSet<>();

    @Inject
    public SysTopicViewController(CoreManager coreManager,
                           ThemeManager themeManager,
                           PubSubTaskFactories pubSubTaskFactories,
                           SysTopicCellControllerFactory sysTopicCellControllerFactory,
                           @Assisted String connectionId) {
        this.coreManager = coreManager;
        this.pubSubTaskFactories = pubSubTaskFactories;
        this.themeManager = themeManager;
        this.sysTopicCellControllerFactory = sysTopicCellControllerFactory;
        this.connectionId = connectionId;
        resources = ResourceBundle.getBundle("org.correomqtt.plugins.systopic.i18n", coreManager.getSettingsManager().getSettings().getCurrentLocale());
    }

    void showAsDialog() throws IOException {
        Map<Object, Object> properties = new HashMap<>();
        properties.put(WindowProperty.WINDOW_TYPE, WindowType.SYSTOPIC);
        properties.put(WindowProperty.CONNECTION_ID, connectionId);
        if (WindowHelper.focusWindowIfAlreadyThere(properties)) {
            return;
        }
        FXMLLoader loader = new FXMLLoader(SysTopicViewController.class.getResource("/org/correomqtt/plugins/systopic/controller/sysTopicsView.fxml"), resources);
        loader.setController(this);
        Parent parent = loader.load();
        showAsDialog((Pane) parent,
                resources.getString("sysTopicsViewControllerTitle") + " " + coreManager.getConnectionManager().getConfig(connectionId).getName(),
                properties,
                event -> onCloseDialog());
    }

    private void onCloseDialog() {
        pubSubTaskFactories.getUnsubscribeFactory()
                .create(connectionId, SubscriptionTransformer.propsToDTO(subscriptionDTO))
                .run();
    }

    @FXML
    private void initialize() {
        ConnectionConfigDTO configDTO = coreManager.getConnectionManager().getConfig(connectionId);
        connectionStatusLabel.setText(configDTO.getUrl() + ":" + configDTO.getPort());
        lastUpdateLabel.setText("Last Update: no update yet");
        sysTopics = FXCollections.observableArrayList(SysTopicPropertiesDTO.extractor());
        listView.setItems(sysTopics);
        listView.setCellFactory(this::createCell);
        pubSubTaskFactories.getSubscribeFactory()
                .create(connectionId, SubscriptionTransformer.propsToDTO(subscriptionDTO))
                .run();
    }

    private ListCell<SysTopicPropertiesDTO> createCell(ListView<SysTopicPropertiesDTO> listView) {
        return sysTopicCellControllerFactory.create(listView);
    }

    @FXML
    void copyToClipboard() {
        ClipboardHelper.addToClipboard(
                plainSysTopics.stream()
                        .sorted(Comparator.comparingInt(st -> {
                            int sortIndex = SysTopic.getSortIndex(st.getTopic());
                            if (st.getSysTopic() != null && st.getSysTopic().isAggregated()) {
                                String parsedComponent = st.getSysTopic().parseComponent(st.getTopic());
                                return sortIndex * 20 + ((parsedComponent == null) ? 0 : Integer.parseInt(parsedComponent));
                            } else {
                                return sortIndex * 20;
                            }
                        }))
                        .map(st -> st.getTopic() + "\t" + st.getPayload())
                        .collect(Collectors.joining("\n"))
        );
    }

    public void onMessageIncoming(@Observes IncomingMessageEvent event) {
        if (!event.getMessageDTO().getTopic().startsWith("$SYS")) {
            return;
        }
        Platform.runLater(() -> insertIncomingMessage(event.getMessageDTO()));
    }

    private void insertIncomingMessage(MessageDTO messageDTO) {
        SysTopic sysTopic = SysTopic.getSysTopicByTopic(messageDTO.getTopic());
        SysTopicPropertiesDTO newSysTopicDTO = SysTopicTransformer.dtoToProps(messageDTO, sysTopic);
        plainSysTopics.remove(newSysTopicDTO);
        plainSysTopics.add(newSysTopicDTO);
        SysTopicPropertiesDTO sysTopicDTO = sysTopics.stream()
                .filter(st ->
                        sysTopic == null && st.getTopic().equals(messageDTO.getTopic()) ||
                                sysTopic == st.getSysTopic())
                .findFirst()
                .orElse(null);
        if (sysTopicDTO == null) {
            sysTopicDTO = newSysTopicDTO;
            sysTopics.add(sysTopicDTO);
            sysTopics.sort(Comparator.comparingInt(st -> SysTopic.getSortIndex(st.getTopic())));
        }
        if (sysTopic != null && sysTopic.isAggregated()) {
            String parsedComponent = sysTopic.parseComponent(messageDTO.getTopic());
            if ("1".equals(parsedComponent)) {
                sysTopicDTO.setMin1(messageDTO.getPayload());
            } else if ("5".equals(parsedComponent)) {
                sysTopicDTO.setMin5(messageDTO.getPayload());
            } else if ("15".equals(parsedComponent)) {
                sysTopicDTO.setMin15(messageDTO.getPayload());
            }
        } else {
            sysTopicDTO.setPayload(messageDTO.getPayload());
        }
        lastUpdateLabel.setText(resources.getString("sysTopicsViewUpdateLabel") + ": " + LocalDateTime.now(ZoneOffset.UTC).toString()); //format
    }

    public void onSubscribedSucceeded(@Observes SubscribeEvent event) {
        if (SYS_TOPIC.equals(event.getSubscriptionDTO().getTopic())) {
            lastUpdateLabel.setText(resources.getString("sysTopicsViewControllerSubscriptionTo")
                    + " "
                    + event.getSubscriptionDTO().getTopic()
                    + " "
                    + resources.getString("sysTopicsViewControllerSucceeded"));
        }
    }

    public void onSubscribedFailed(@Observes SubscribeFailedEvent event) {
        if (SYS_TOPIC.equals(event.getSubscriptionDTO().getTopic())) {
            lastUpdateLabel.setText(resources.getString("sysTopicsViewControllerSubscriptionTo")
                    + " "
                    + event.getSubscriptionDTO().getTopic()
                    + " "
                    + resources.getString("sysTopicsViewControllerFailed"));
        }
    }

    void showAsDialog(Pane Pane, String title, Map<Object, Object> windowProperties, final EventHandler<WindowEvent> closeHandler) {
        String cssPath = themeManager.getCssPath();
        Scene scene = new Scene(Pane);
        scene.setFill(themeManager.getActiveTheme().getBackgroundColor());
        if (cssPath != null) scene.getStylesheets().add(cssPath);
        Stage stage = new Stage();
        stage.setResizable(true);
        stage.setAlwaysOnTop(false);
        stage.setTitle(title);
        stage.setScene(scene);
        stage.show();
        if (closeHandler != null) {
            stage.getScene().getWindow().addEventFilter(WindowEvent.WINDOW_CLOSE_REQUEST, closeHandler);
        }
        stage.getScene().getWindow().getProperties().putAll(windowProperties);
    }
}

