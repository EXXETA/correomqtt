package org.correomqtt.gui.controller;

import com.exxeta.correomqtt.business.dispatcher.SubscribeDispatcher;
import com.exxeta.correomqtt.business.dispatcher.SubscribeObserver;
import com.exxeta.correomqtt.business.model.ConnectionConfigDTO;
import com.exxeta.correomqtt.business.model.MessageDTO;
import com.exxeta.correomqtt.business.model.Qos;
import com.exxeta.correomqtt.business.model.SubscriptionDTO;
import com.exxeta.correomqtt.business.model.SysTopic;
import com.exxeta.correomqtt.business.utils.ConnectionHolder;
import com.exxeta.correomqtt.gui.business.TaskFactory;
import com.exxeta.correomqtt.gui.cell.SysTopicCell;
import com.exxeta.correomqtt.gui.helper.ClipboardHelper;
import com.exxeta.correomqtt.gui.model.SubscriptionPropertiesDTO;
import com.exxeta.correomqtt.gui.model.SysTopicPropertiesDTO;
import com.exxeta.correomqtt.gui.model.WindowProperty;
import com.exxeta.correomqtt.gui.model.WindowType;
import com.exxeta.correomqtt.gui.transformer.SysTopicTransformer;
import com.exxeta.correomqtt.gui.utils.WindowHelper;
import javafx.application.Platform;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.stage.WindowEvent;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.time.LocalDateTime;
import java.time.ZoneOffset;
import java.util.Comparator;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Map;
import java.util.ResourceBundle;
import java.util.Set;
import java.util.stream.Collectors;


public class SysTopicViewController extends BaseConnectionController implements SubscribeObserver {

    private static final Logger LOGGER = LoggerFactory.getLogger(SysTopicViewController.class);
    private static final String SYS_TOPIC = "$SYS/#";
    private static ResourceBundle resources;

    private final SubscriptionPropertiesDTO subscriptionDTO = SubscriptionPropertiesDTO.builder()
                                                                                       .topic(SYS_TOPIC)
                                                                                       .qos(Qos.AT_LEAST_ONCE)
                                                                                       .hidden(true)
                                                                                       .build();

    @FXML
    private Label connectionStatusLabel;
    @FXML
    private Label lastUpdateLabel;
    @FXML
    private ListView<SysTopicPropertiesDTO> listView;

    private ObservableList<SysTopicPropertiesDTO> sysTopics;

    private Set<SysTopicPropertiesDTO> plainSysTopics = new HashSet<>();

    protected SysTopicViewController(String connectionId) {
        super(connectionId);


        SubscribeDispatcher.getInstance().addObserver(this);
    }

    public static void showAsDialog(final String connectionId) {

        Map<Object, Object> properties = new HashMap<>();
        properties.put(WindowProperty.WINDOW_TYPE, WindowType.SYSTOPIC);
        properties.put(WindowProperty.CONNECTION_ID, connectionId);

        if(WindowHelper.focusWindowIfAlreadyThere(properties)){
            return;
        }

        LoaderResult<SysTopicViewController> result = load(SysTopicViewController.class, "sysTopicsView.fxml", connectionId);
        resources = result.getResourceBundle();
        showAsDialog(result,
                resources.getString("sysTopicsViewControllerTitle") + " " + ConnectionHolder.getInstance().getConfig(connectionId).getName(),
                     properties,
                     true,
                     false, event -> result.getController().onCloseDialog(event), null);
    }

    private void onCloseDialog(WindowEvent event) {
        TaskFactory.unsubscribe(getConnectionId(), subscriptionDTO);
    }

    @FXML
    private void initialize() {
        ConnectionConfigDTO configDTO = ConnectionHolder.getInstance().getConfig(getConnectionId());
        connectionStatusLabel.setText(configDTO.getUrl() + ":" + configDTO.getPort());

        lastUpdateLabel.setText("Last Update: no update yet");
        sysTopics = FXCollections.observableArrayList(SysTopicPropertiesDTO.extractor());

        listView.setItems(sysTopics);
        listView.setCellFactory(this::createCell);

        TaskFactory.subscribe(getConnectionId(), subscriptionDTO);
    }

    private ListCell<SysTopicPropertiesDTO> createCell(ListView<SysTopicPropertiesDTO> listView) {
        return new SysTopicCell(listView);
    }

    @FXML
    public void copyToClipboard() {

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

    @Override
    public void onMessageIncoming(MessageDTO messageDTO, SubscriptionDTO subscriptionDTO) {

        if (!messageDTO.getTopic().startsWith("$SYS")) {
            return;
        }

        Platform.runLater(() -> insertIncomingMessage(messageDTO));
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

    @Override
    public void onSubscribedSucceeded(SubscriptionDTO subscriptionDTO) {
        if (SYS_TOPIC.equals(subscriptionDTO.getTopic())) {
            lastUpdateLabel.setText(resources.getString("sysTopicsViewControllerSubscriptionTo") + " " + subscriptionDTO.getTopic() + " " + resources.getString("sysTopicsViewControllerSucceeded"));
        }
    }

    @Override
    public void onSubscribedCanceled(SubscriptionDTO subscriptionDTO) {
        if (SYS_TOPIC.equals(subscriptionDTO.getTopic())) {
            lastUpdateLabel.setText(resources.getString("sysTopicsViewControllerSubscriptionTo") + " " + subscriptionDTO.getTopic() + " " + resources.getString("sysTopicsViewControllerCancelled"));
        }
    }

    @Override
    public void onSubscribedFailed(SubscriptionDTO subscriptionDTO, Throwable exception) {
        if (SYS_TOPIC.equals(subscriptionDTO.getTopic())) {
            lastUpdateLabel.setText(resources.getString("sysTopicsViewControllerSubscriptionTo") + " " + subscriptionDTO.getTopic() + " " + resources.getString("sysTopicsViewControllerFailed"));
        }
    }
}

