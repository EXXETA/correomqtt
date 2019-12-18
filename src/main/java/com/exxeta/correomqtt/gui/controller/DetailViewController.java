package com.exxeta.correomqtt.gui.controller;

import com.exxeta.correomqtt.business.dispatcher.*;
import com.exxeta.correomqtt.business.model.MessageDTO;
import com.exxeta.correomqtt.business.model.MessageType;
import com.exxeta.correomqtt.business.services.ConfigService;
import com.exxeta.correomqtt.gui.contextmenu.DetailContextMenu;
import com.exxeta.correomqtt.gui.contextmenu.DetailContextMenuDelegate;
import com.exxeta.correomqtt.gui.formats.Format;
import com.exxeta.correomqtt.gui.formats.Plain;
import com.exxeta.correomqtt.gui.menuitem.TaskMenuItem;
import com.exxeta.correomqtt.gui.model.MessagePropertiesDTO;
import com.exxeta.correomqtt.gui.model.Search;
import com.exxeta.correomqtt.gui.model.WindowProperty;
import com.exxeta.correomqtt.gui.model.WindowType;
import com.exxeta.correomqtt.gui.utils.MessageUtils;
import com.exxeta.correomqtt.gui.utils.WindowHelper;
import com.exxeta.correomqtt.plugin.manager.MessageValidator;
import com.exxeta.correomqtt.plugin.manager.PluginProtocolTask;
import com.exxeta.correomqtt.plugin.manager.PluginSystem;
import com.exxeta.correomqtt.plugin.model.MessageExtensionDTO;
import com.exxeta.correomqtt.plugin.spi.DetailViewFormatHook;
import com.exxeta.correomqtt.plugin.spi.DetailViewHook;
import com.exxeta.correomqtt.plugin.spi.DetailViewManipulatorHook;
import com.exxeta.correomqtt.plugin.spi.MessageValidatorHook;
import javafx.application.Platform;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.collections.ObservableList;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.geometry.Pos;
import javafx.scene.Node;
import javafx.scene.control.*;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Pane;
import javafx.scene.layout.Priority;
import javafx.scene.layout.VBox;
import javafx.stage.Stage;
import org.fxmisc.flowless.VirtualizedScrollPane;
import org.fxmisc.richtext.CodeArea;
import org.pf4j.PluginState;
import org.pf4j.PluginStateListener;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.util.*;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import java.util.regex.PatternSyntaxException;


public class DetailViewController extends BaseConnectionController implements
        DetailContextMenuDelegate,
        ImportMessageObserver,
        ExportMessageObserver,
        ApplicationLifecycleObserver {

    private static final Logger LOGGER = LoggerFactory.getLogger(DetailViewController.class);

    private static final String CHECK_SOLID_CLASS = "check-solid";
    private static ResourceBundle resources;
    private final BooleanProperty inlineViewProperty;
    private final DetailViewDelegate delegate;

    @FXML
    private Button detailViewSaveButton;
    @FXML
    private SplitMenuButton manipulateSelectionButton;
    @FXML
    private Label detailViewTopicLabel;
    @FXML
    private Label detailViewTime;
    @FXML
    private HBox detailViewNodeBox;
    @FXML
    private Label detailViewValid;
    @FXML
    private Label detailViewInvalid;
    @FXML
    private Label detailViewRetained;
    @FXML
    private Label detailViewQos;
    @FXML
    private Pane detailViewScrollPane;
    @FXML
    private CodeArea codeArea;
    @FXML
    private VBox detailViewVBox;
    @FXML
    private HBox detailViewToolBar;
    @FXML
    private ToggleButton detailViewFormatToggleButton;
    @FXML
    private Button detailViewSearchButton;
    @FXML
    private HBox detailViewSearchHBox;
    @FXML
    private TextField searchTextField;
    @FXML
    private Button selectPreviousResult;
    @FXML
    private Button selectNextResult;
    @FXML
    private Label resultsLabel;
    @FXML
    private MenuButton searchMenuButton;
    @FXML
    private MenuItem ignoreCaseMenuItem;
    @FXML
    private MenuItem regexMenuItem;
    @FXML
    private Label closeLabel;
    @FXML
    private Label noPayloadLabel;
    @FXML
    private VBox messageGroup;
    @FXML
    private Label emptyLabel;
    @FXML
    private VBox metaHolder;
    private List<Search> results;
    private int currentSearchResult;
    private String currentSearchString = null;
    private String codeAreaText;

    private MessagePropertiesDTO messageDTO;
    private DetailContextMenu contextMenu;
    private PluginProtocolTask<DetailViewManipulatorHook> lastManipulatorTask;

    private DetailViewController(String connectionId, DetailViewDelegate delegate, boolean isInlineView) {
        super(connectionId);
        this.delegate = delegate;
        inlineViewProperty = new SimpleBooleanProperty(isInlineView);
        ExportMessageDispatcher.getInstance().addObserver(this);
        ImportMessageDispatcher.getInstance().addObserver(this);
        ApplicationLifecycleDispatcher.getInstance().addObserver(this);
    }

    private static LoaderResult<DetailViewController> load(final String connectionId, DetailViewDelegate delegate, final boolean isInlineView) {
        return load(DetailViewController.class,
                "detailView.fxml",
                () -> new DetailViewController(connectionId, delegate, isInlineView));
    }

    public static LoaderResult<DetailViewController> load(MessagePropertiesDTO messageDTO, String connectionId, DetailViewDelegate delegate, boolean isInlineView) {
        LoaderResult<DetailViewController> result = load(connectionId, delegate, isInlineView);
        result.getController().setMessage(messageDTO);
        resources = result.getResourceBundle();

        if (!isInlineView) {
            result.getController().disableInlineView();
        }

        return result;
    }

    static void showAsDialog(MessagePropertiesDTO messageDTO, String connectionId, DetailViewDelegate delegate) {

        Map<Object, Object> properties = new HashMap<>();
        properties.put(WindowProperty.WINDOW_TYPE, WindowType.DETAIL);
        properties.put(WindowProperty.CONNECTION_ID, connectionId);
        properties.put(WindowProperty.MESSAGE_ID, messageDTO.getMessageId());

        if (WindowHelper.focusWindowIfAlreadyThere(properties)) {
            return;
        }

        LoaderResult<DetailViewController> result = load(messageDTO, connectionId, delegate, false);

        String title;
        if (messageDTO.getMessageType() == MessageType.INCOMING) {
            title = resources.getString("detailViewControllerIncomingTitle");
        } else if (messageDTO.getMessageType() == MessageType.OUTGOING) {
            title = resources.getString("detailViewControllerOutgoingTitle");
        } else {
            title = resources.getString("detailViewControllerTitle");
        }

        showAsDialog(result, title, properties, true, false, null, null);
    }

    private void disableInlineView() {
        inlineViewProperty.setValue(false);
    }

    @FXML
    private void initialize() {
        detailViewVBox.getStyleClass().add(ConfigService.getInstance().getThemeSettings().getActiveTheme().getIconMode());
        searchTextField.textProperty().addListener((observable, oldValue, newValue) -> performSearch(newValue));
        searchTextField.addEventHandler(KeyEvent.KEY_PRESSED, event -> {
            if (KeyCode.ESCAPE == event.getCode()) {
                closeSearch();
            } else if (KeyCode.ENTER == event.getCode()) {
                selectNextResult();

            }
        });

        detailViewVBox.addEventHandler(KeyEvent.KEY_PRESSED, event -> {
            if (event.isShortcutDown() && KeyCode.F == event.getCode()) {
                toggleSearchBar();
            }
        });

        detailViewScrollPane.prefWidthProperty().bind(detailViewVBox.widthProperty());
        detailViewScrollPane.prefHeightProperty().bind(detailViewVBox.heightProperty());

        contextMenu = new DetailContextMenu(this);

        metaHolder.setOnContextMenuRequested(event -> {
            if (messageDTO != null) {
                contextMenu.setObject(messageDTO);
                contextMenu.show(metaHolder, event.getScreenX(), event.getScreenY());
            }
        });

        detailViewFormatToggleButton.setOnMouseClicked(mouseEvent -> {
            autoFormatPayload(messageDTO.getPayload(), detailViewFormatToggleButton.isSelected());
            showSearchResult();
        });

        initializeManipulateButton();
        registerPluginStateListener();
    }

    private void registerPluginStateListener() {
        PluginSystem.getInstance().addPluginStateListener(pluginStateListener);
    }

    private PluginStateListener pluginStateListener = event -> {
        if (event.getPluginState().equals(PluginState.STOPPED) || event.getPluginState().equals(PluginState.STARTED)) {
            initializeManipulateButton();
        }
    };

    private void initializeManipulateButton() {
        manipulateSelectionButton.getItems().clear();
        lastManipulatorTask = null;
        manipulateSelectionButton.setText("Manipulate");

        List<PluginProtocolTask<DetailViewManipulatorHook>> tasks = PluginSystem.getInstance().getTasks(DetailViewManipulatorHook.class);
        tasks.forEach(p -> {
            TaskMenuItem<DetailViewManipulatorHook> menuItem = new TaskMenuItem<>(p);
            menuItem.setOnAction(this::onManipulateMessageSelected);
            manipulateSelectionButton.getItems().add(menuItem);
        });
        manipulateSelectionButton.setOnAction(this::onManipulateMessageClicked);
        manipulateSelectionButton.setVisible(!tasks.isEmpty());
        manipulateSelectionButton.setManaged(!tasks.isEmpty());
    }

    @FXML
    private void changeIgnoreCase() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Clicked on changeIgnoreCase: {}", getConnectionId());
        }

        ConfigService.getInstance().getSettings().setUseIgnoreCase(!ConfigService.getInstance().getSettings().isUseIgnoreCase());
        ConfigService.getInstance().saveSettings();
        this.searchMenuButton.getItems().remove(ignoreCaseMenuItem);
        if (ConfigService.getInstance().getSettings().isUseIgnoreCase()) {
            ignoreCaseMenuItem.getStyleClass().add(CHECK_SOLID_CLASS);
        } else {
            ignoreCaseMenuItem.getStyleClass().remove(CHECK_SOLID_CLASS);
        }
        this.searchMenuButton.getItems().add(0, ignoreCaseMenuItem);
        currentSearchString = null;
        currentSearchResult = 0;
        showSearchResult();
    }

    @FXML
    private void changeRegex() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Clicked on changeRegex: {}", getConnectionId());
        }

        ConfigService.getInstance().getSettings().setUseRegexForSearch(!ConfigService.getInstance().getSettings().isUseRegexForSearch());
        ConfigService.getInstance().saveSettings();
        this.searchMenuButton.getItems().remove(regexMenuItem);
        if (ConfigService.getInstance().getSettings().isUseRegexForSearch()) {
            regexMenuItem.getStyleClass().add(CHECK_SOLID_CLASS);
        } else {
            regexMenuItem.getStyleClass().remove(CHECK_SOLID_CLASS);
        }
        this.searchMenuButton.getItems().add(1, regexMenuItem);
        currentSearchString = null;
        currentSearchResult = 0;
        showSearchResult();
    }

    void setMessage(MessagePropertiesDTO messageDTO) {
        this.messageDTO = messageDTO;
        if (messageDTO == null) {
            showNoMessage();
        } else {
            showMessage();
        }
    }

    private void showMessage() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Show message in DetailView: {}", getConnectionId());
        }

        emptyLabel.setVisible(false);
        emptyLabel.setManaged(false);
        messageGroup.setVisible(true);
        messageGroup.setManaged(true);

        detailViewTopicLabel.setText(messageDTO.getTopic());
        detailViewTime.setText(messageDTO.getDateTime().toString()); //TODO formatter

        detailViewSaveButton.setDisable(false);

        executeOnOpenDetailViewExtensions();

        validateMessage(messageDTO.getTopic(), messageDTO.getPayload());

        detailViewRetained.setManaged(messageDTO.isRetained());
        detailViewRetained.setVisible(messageDTO.isRetained());

        detailViewQos.setText(messageDTO.getQos().toString());

        if (ConfigService.getInstance().getSettings().isUseIgnoreCase()) {
            ignoreCaseMenuItem.getStyleClass().add(CHECK_SOLID_CLASS);
        }
        if (ConfigService.getInstance().getSettings().isUseRegexForSearch()) {
            regexMenuItem.getStyleClass().add(CHECK_SOLID_CLASS);
        }

        detailViewSaveButton.setOnMouseClicked(this::saveMessage);

        if (messageDTO.getPayload().isEmpty()) {
            clearPayload();
        } else {
            updatePayload(messageDTO.getPayload());
        }
    }

    private void executeOnOpenDetailViewExtensions() {
        detailViewNodeBox.getChildren().clear();
        PluginSystem.getInstance().getExtensions(DetailViewHook.class).forEach(p -> {
            HBox pluginBox = new HBox();
            pluginBox.setAlignment(Pos.CENTER_RIGHT);
            HBox.setHgrow(pluginBox, Priority.ALWAYS);
            detailViewNodeBox.getChildren().add(pluginBox);
            p.onOpenDetailView(new MessageExtensionDTO(messageDTO), pluginBox);
        });
    }

    private void validateMessage(String topic, String payload) {
        detailViewValid.setVisible(false);
        detailViewValid.setManaged(false);
        detailViewInvalid.setVisible(false);
        detailViewInvalid.setManaged(false);

        MessageValidatorHook.Validation validation = MessageValidator.validateMessage(topic, payload);
        if (validation != null) {
            updateValidatorLabel(detailViewValid, validation.isValid(), validation.getTooltip());
            updateValidatorLabel(detailViewInvalid, !validation.isValid(), validation.getTooltip());
        }
    }

    private void updateValidatorLabel(Label label, boolean isVisible, String tooltip) {
        label.setVisible(isVisible);
        label.setManaged(isVisible);
        label.setTooltip(new Tooltip(tooltip));
    }

    private void onManipulateMessageClicked(ActionEvent actionEvent) {
        if (lastManipulatorTask != null) {
            manipulateMessage(lastManipulatorTask);
        }
    }

    private void onManipulateMessageSelected(ActionEvent actionEvent) {
        PluginProtocolTask<DetailViewManipulatorHook> manipulatorTask = ((TaskMenuItem) actionEvent.getSource()).getTask();
        manipulateMessage(manipulatorTask);
        manipulateSelectionButton.setText(manipulatorTask.getId());
        this.lastManipulatorTask = manipulatorTask;
    }

    private void manipulateMessage(PluginProtocolTask<DetailViewManipulatorHook> manipulatorTask) {
        IndexRange range = getSelectionRange();

        byte[] selection = codeArea.getText(range).getBytes();
        for (DetailViewManipulatorHook hook : manipulatorTask.getTasks()) {
            selection = hook.manipulate(selection);
        }

        String manipulatedSelection = new String(selection);
        codeArea.replaceText(range, manipulatedSelection);
        codeArea.selectRange(range.getStart(), range.getStart() + manipulatedSelection.length());
        detailViewFormatToggleButton.setSelected(false);

        if (messageDTO != null) {
            validateMessage(messageDTO.getTopic(), codeArea.getText());
            autoFormatPayload(codeArea.getText(), true);
        }
    }

    private IndexRange getSelectionRange() {
        IndexRange selection = codeArea.getSelection();
        if (selection.getLength() == 0) {
            return new IndexRange(0, codeArea.getLength());
        }
        return selection;
    }

    private void showNoMessage() {

        detailViewSaveButton.setDisable(true);
        detailViewFormatToggleButton.setDisable(true);
        detailViewSearchButton.setDisable(true);

        closeSearch();

        emptyLabel.setVisible(true);
        emptyLabel.setManaged(true);
        messageGroup.setVisible(false);
        messageGroup.setManaged(false);
    }

    private void updatePayload(String payload) {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Updating payload: {}", getConnectionId());
        }

        detailViewSearchButton.setDisable(false);
        closeSearch();

        noPayloadLabel.setManaged(false);
        noPayloadLabel.setVisible(false);
        detailViewScrollPane.setManaged(true);
        detailViewScrollPane.setVisible(true);
        detailViewScrollPane.getChildren().add(new VirtualizedScrollPane<>(codeArea));

        codeArea.prefWidthProperty().bind(detailViewScrollPane.widthProperty());
        codeArea.prefHeightProperty().bind(detailViewScrollPane.heightProperty());

        codeArea.setEditable(false);

        Format format = autoFormatPayload(payload, true);
        detailViewFormatToggleButton.setSelected(format.isFormatable());
        detailViewFormatToggleButton.setDisable(!format.isFormatable());
    }

    private void clearPayload() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Clearing payload: {}", getConnectionId());
        }

        detailViewSearchButton.setDisable(true);
        closeSearch();
        noPayloadLabel.setManaged(true);
        noPayloadLabel.setVisible(true);
        detailViewScrollPane.setManaged(false);
        detailViewScrollPane.setVisible(false);
        detailViewFormatToggleButton.setSelected(false);
        detailViewFormatToggleButton.setDisable(true);
        detailViewSearchButton.setDisable(true);
        noPayloadLabel.prefWidthProperty().bind(detailViewVBox.widthProperty());
        noPayloadLabel.prefHeightProperty().bind(detailViewVBox.heightProperty());
    }

    private void saveMessage(MouseEvent event) {

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Clicked on save message: {}", getConnectionId());
        }

        Stage stage = (Stage) detailViewVBox.getScene().getWindow();
        MessageUtils.saveMessage(getConnectionId(), messageDTO, stage);
    }

    private Format autoFormatPayload(final String payload, boolean doFormatting) {

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Auto formatting payload: {}", getConnectionId());
        }

        Format foundFormat;
        if (doFormatting) {
            // Find the first format that is valid.
            ArrayList<Format> availableFormats = new ArrayList<>(PluginSystem.getInstance().getExtensions(DetailViewFormatHook.class));
            availableFormats.add(new Plain());
            foundFormat = availableFormats.stream()
                    .filter(Objects::nonNull)
                    .filter(format -> {
                                format.setText(payload);
                                return format.isValid();
                            }
                    )
                    .findFirst()
                    .orElseThrow(() -> new IllegalArgumentException("Plain format did not match."));
        } else {
            foundFormat = new Plain();
            foundFormat.setText(payload);
        }

        codeArea.clear();
        codeArea.replaceText(0, 0, foundFormat.getPrettyString());
        codeArea.setStyleSpans(0, foundFormat.getFxSpans());
        return foundFormat;
    }

    private void showSearchResult() {

        currentSearchString = searchTextField.textProperty().get();

        results = new ArrayList<>();

        if (!currentSearchString.isEmpty()) {
            codeAreaText = codeArea.getText();

            boolean ignoreCase = ConfigService.getInstance().getSettings().isUseIgnoreCase();
            boolean regex = ConfigService.getInstance().getSettings().isUseRegexForSearch();

            String finalSearchString;
            if (regex) {
                finalSearchString = currentSearchString;
            } else {
                finalSearchString = Pattern.quote(currentSearchString);
            }

            try {
                Pattern searchPattern = Pattern.compile(finalSearchString, ignoreCase ? Pattern.CASE_INSENSITIVE : 0x00);
                Matcher matcher = searchPattern.matcher(codeAreaText);

                while (matcher.find()) {
                    results.add(results.size(), new Search(matcher.start(), matcher.end()));
                }
            } catch (PatternSyntaxException e) {
                LOGGER.debug("Invalid pattern: {}", e.getMessage());
            }
        }

        if (!results.isEmpty()) {
            currentSearchResult = 0;
            selectPreviousResult.setDisable(false);
            selectNextResult.setDisable(false);
        } else {
            resultsLabel.setText(null);
            selectPreviousResult.setDisable(true);
            selectNextResult.setDisable(true);
            codeArea.deselect();
        }

        updateSearchResult();
    }

    private void performSearch(String newValue) {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Performing search: {}", getConnectionId());
        }

        if (newValue != null && !newValue.isEmpty() && newValue.equals(currentSearchString)) {
            selectNextResult();
            return;
        }
        showSearchResult();
    }

    private void updateSearchResult() {
        if (results == null || results.isEmpty()) {
            resultsLabel.setText(resources.getString("detailViewControllerNoResults"));
            return;
        }

        codeArea.selectRange(results.get(currentSearchResult).getStartIndex(), results.get(currentSearchResult).getEndIndex());
        resultsLabel.setText(currentSearchResult + 1 + " " + resources.getString("detailViewControllerOf") + " "
                + results.size() + " " + resources.getString("detailViewControllerMatches"));

        /* How to display the current search always in sight.
         * 1. Scroll to left.
         * 2. Move cursor to end of selection.
         * 3. Adjust view port to cursor.
         */
        codeArea.scrollXToPixel(0);
        codeArea.displaceCaret(results.get(currentSearchResult).getEndIndex());
        codeArea.requestFollowCaret();

        // after changes e.g. ignore case one want to continue the search e.g. hit enter
        searchTextField.requestFocus();
    }

    @FXML
    private void selectNextResult() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Clicked on next result: {}", getConnectionId());
        }

        if (results == null) {
            currentSearchResult = 0;
        } else {
            currentSearchResult += 1;
            if (currentSearchResult == results.size()) {
                currentSearchResult = 0;
            }
        }
        updateSearchResult();
    }

    @FXML
    private void selectPreviousResult() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Clicked on previous result: {}", getConnectionId());
        }

        if (results == null) {
            currentSearchResult = 0;
        } else {
            currentSearchResult -= 1;
            if (currentSearchResult == -1) {
                currentSearchResult = results.size() - 1;
            }
        }
        updateSearchResult();
    }

    public void toggleSearchBar() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Toggling search bar: {}", getConnectionId());
        }
        if (detailViewSearchButton.isDisabled()) {
            return;
        }

        if (detailViewToolBar.getChildren().contains(detailViewSearchHBox)) {
            closeSearch();
        } else {
            showSearch(true);
        }
    }

    private void showSearch(boolean focus) {
        detailViewToolBar.getChildren().remove(detailViewSearchButton);
        if (!detailViewToolBar.getChildren().contains(detailViewSearchHBox)) {
            detailViewToolBar.getChildren().add(detailViewSearchHBox);
        }
        if (focus) {
            searchTextField.requestFocus();
        }
    }

    @FXML
    private void closeSearch() {
        ObservableList<Node> barChildren = detailViewToolBar.getChildren();
        barChildren.remove(detailViewSearchHBox);
        if (!barChildren.contains(detailViewSearchButton)) {
            barChildren.add(detailViewSearchButton);
        }
        resultsLabel.setText("");
        searchTextField.setText("");
        codeArea.deselect();

        //in order to make shortcuts work again, the main pane must be focused
        detailViewVBox.requestFocus();
    }

    Node getMainNode() {
        return detailViewVBox;
    }

    @Override
    public void showDetailsInSeparateWindow(MessagePropertiesDTO messageDTO) {
        showAsDialog(messageDTO, getConnectionId(), delegate);
    }

    @Override
    public void setUpToForm(MessagePropertiesDTO messageDTO) {
        delegate.setUpToForm(messageDTO);
    }

    @Override
    public BooleanProperty isInlineView() {
        return inlineViewProperty;
    }

    @Override
    public void onExportStarted(File file, MessageDTO messageDTO) {
        Platform.runLater(() -> {
            detailViewVBox.setDisable(true);
        });
    }

    @Override
    public void onExportSucceeded() {
        Platform.runLater(() -> {
            detailViewVBox.setDisable(false);
        });
    }

    @Override
    public void onExportCancelled(File file, MessageDTO messageDTO) {
        Platform.runLater(() -> {
            detailViewVBox.setDisable(false);
        });
    }

    @Override
    public void onExportFailed(File file, MessageDTO messageDTO, Throwable exception) {
        Platform.runLater(() -> {
            detailViewVBox.setDisable(false);
        });
    }

    @Override
    public void onImportStarted(File file) {
        detailViewVBox.setDisable(true);
    }

    @Override
    public void onImportSucceeded(MessageDTO messageDTO) {
        detailViewVBox.setDisable(false);
    }

    @Override
    public void onImportCancelled(File file) {
        detailViewVBox.setDisable(false);
    }

    @Override
    public void onImportFailed(File file, Throwable exception) {
        detailViewVBox.setDisable(false);
    }

    @Override
    public void onShutdown() {
        PluginSystem.getInstance().removePluginStateListener(pluginStateListener);
    }
}
