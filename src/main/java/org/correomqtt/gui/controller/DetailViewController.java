package org.correomqtt.gui.controller;

import javafx.scene.control.Button;
import javafx.scene.control.IndexRange;
import javafx.scene.control.Label;
import javafx.scene.control.MenuButton;
import javafx.scene.control.MenuItem;
import javafx.scene.control.SplitMenuButton;
import javafx.scene.control.TextField;
import javafx.scene.control.ToggleButton;
import javafx.scene.control.Tooltip;
import org.correomqtt.business.dispatcher.ExportMessageDispatcher;
import org.correomqtt.business.dispatcher.ExportMessageObserver;
import org.correomqtt.business.dispatcher.ImportMessageDispatcher;
import org.correomqtt.business.dispatcher.ImportMessageObserver;
import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.model.MessageType;
import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.business.utils.AutoFormatPayload;
import org.correomqtt.gui.contextmenu.DetailContextMenu;
import org.correomqtt.gui.contextmenu.DetailContextMenuDelegate;
import org.correomqtt.gui.formats.Format;
import org.correomqtt.gui.formats.Plain;
import org.correomqtt.gui.menuitem.TaskMenuItem;
import org.correomqtt.gui.model.MessagePropertiesDTO;
import org.correomqtt.gui.model.Search;
import org.correomqtt.gui.model.WindowProperty;
import org.correomqtt.gui.model.WindowType;
import org.correomqtt.gui.utils.MessageUtils;
import org.correomqtt.gui.utils.WindowHelper;
import org.correomqtt.plugin.manager.MessageValidator;
import org.correomqtt.plugin.manager.PluginManager;
import org.correomqtt.plugin.manager.Task;
import org.correomqtt.plugin.model.MessageExtensionDTO;
import org.correomqtt.plugin.spi.DetailViewFormatHook;
import org.correomqtt.plugin.spi.DetailViewHook;
import org.correomqtt.plugin.spi.DetailViewManipulatorHook;
import org.correomqtt.plugin.spi.MessageValidatorHook;
import javafx.application.Platform;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.collections.ObservableList;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.geometry.Pos;
import javafx.scene.Node;
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
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.ResourceBundle;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import java.util.regex.PatternSyntaxException;


public class DetailViewController extends BaseConnectionController implements
        DetailContextMenuDelegate,
        ImportMessageObserver,
        ExportMessageObserver {

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
    private Button detailViewRevertManipulationButton;
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
    private Task<DetailViewManipulatorHook> lastManipulatorTask;

    private DetailViewController(String connectionId, DetailViewDelegate delegate, boolean isInlineView) {
        super(connectionId);
        this.delegate = delegate;
        inlineViewProperty = new SimpleBooleanProperty(isInlineView);
        ExportMessageDispatcher.getInstance().addObserver(this);
        ImportMessageDispatcher.getInstance().addObserver(this);
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
        detailViewVBox.getStyleClass().add(SettingsProvider.getInstance().getIconModeCssClass());
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
            AutoFormatPayload.autoFormatPayload(messageDTO.getPayload(), detailViewFormatToggleButton.isSelected(), getConnectionId(), codeArea);
            showSearchResult();
        });

        initializeManipulation();
    }

    private void initializeManipulation() {
        manipulateSelectionButton.getItems().clear();
        lastManipulatorTask = null;
        manipulateSelectionButton.setText("Manipulate");

        List<Task<DetailViewManipulatorHook>> tasks = PluginManager.getInstance().getTasks(DetailViewManipulatorHook.class);
        tasks.forEach(p -> {
            TaskMenuItem<DetailViewManipulatorHook> menuItem = new TaskMenuItem<>(p);
            menuItem.setOnAction(this::onManipulateMessageSelected);
            manipulateSelectionButton.getItems().add(menuItem);
        });
        manipulateSelectionButton.setOnMouseClicked(this::onManipulateMessageClicked);
        manipulateSelectionButton.setVisible(!tasks.isEmpty());
        manipulateSelectionButton.setManaged(!tasks.isEmpty());

        detailViewRevertManipulationButton.setOnMouseClicked(e -> showMessage());
    }

    @FXML
    private void changeIgnoreCase() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Clicked on changeIgnoreCase: {}", getConnectionId());
        }

        SettingsProvider.getInstance().getSettings().setUseIgnoreCase(!SettingsProvider.getInstance().getSettings().isUseIgnoreCase());
        SettingsProvider.getInstance().saveSettings(false);
        this.searchMenuButton.getItems().remove(ignoreCaseMenuItem);
        if (SettingsProvider.getInstance().getSettings().isUseIgnoreCase()) {
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

        SettingsProvider.getInstance().getSettings().setUseRegexForSearch(!SettingsProvider.getInstance().getSettings().isUseRegexForSearch());
        SettingsProvider.getInstance().saveSettings(false);
        this.searchMenuButton.getItems().remove(regexMenuItem);
        if (SettingsProvider.getInstance().getSettings().isUseRegexForSearch()) {
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

        detailViewRevertManipulationButton.setDisable(true);

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

        if (SettingsProvider.getInstance().getSettings().isUseIgnoreCase()) {
            ignoreCaseMenuItem.getStyleClass().add(CHECK_SOLID_CLASS);
        }
        if (SettingsProvider.getInstance().getSettings().isUseRegexForSearch()) {
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
        PluginManager.getInstance().getExtensions(DetailViewHook.class).forEach(p -> {
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

    private void onManipulateMessageClicked(MouseEvent event) {
        if (lastManipulatorTask != null) {
            manipulateMessage(lastManipulatorTask);
        }
    }

    private void onManipulateMessageSelected(ActionEvent actionEvent) {
        Task<DetailViewManipulatorHook> manipulatorTask = ((TaskMenuItem) actionEvent.getSource()).getTask();
        manipulateMessage(manipulatorTask);
        manipulateSelectionButton.setText(manipulatorTask.getId());
        this.lastManipulatorTask = manipulatorTask;
    }

    private void manipulateMessage(Task<DetailViewManipulatorHook> manipulatorTask) {
        detailViewRevertManipulationButton.setDisable(false);

        IndexRange range = getSelectionRange();

        byte[] selection = codeArea.getText(range).getBytes();
        for (DetailViewManipulatorHook hook : manipulatorTask.getTasks()) {
            selection = hook.manipulate(selection);
        }

        codeArea.replaceText(range, new String(selection));
        detailViewFormatToggleButton.setSelected(false);

        if (messageDTO != null) {
            validateMessage(messageDTO.getTopic(), codeArea.getText());
            AutoFormatPayload.autoFormatPayload(codeArea.getText(), true, getConnectionId(), codeArea);
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

        Format format = AutoFormatPayload.autoFormatPayload(payload, true, getConnectionId(), codeArea);
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

    private void showSearchResult() {

        currentSearchString = searchTextField.textProperty().get();

        results = new ArrayList<>();

        if (!currentSearchString.isEmpty()) {
            codeAreaText = codeArea.getText();

            boolean ignoreCase = SettingsProvider.getInstance().getSettings().isUseIgnoreCase();
            boolean regex = SettingsProvider.getInstance().getSettings().isUseRegexForSearch();

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
}
