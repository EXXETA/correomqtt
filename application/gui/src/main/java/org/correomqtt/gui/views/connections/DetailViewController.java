package org.correomqtt.gui.views.connections;

import javafx.application.Platform;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.collections.ObservableList;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.geometry.Pos;
import javafx.scene.Node;
import javafx.scene.control.Button;
import javafx.scene.control.IndexRange;
import javafx.scene.control.Label;
import javafx.scene.control.MenuButton;
import javafx.scene.control.SplitMenuButton;
import javafx.scene.control.TextField;
import javafx.scene.control.ToggleButton;
import javafx.scene.control.Tooltip;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Pane;
import javafx.scene.layout.Priority;
import javafx.scene.layout.VBox;
import javafx.stage.Stage;
import org.correomqtt.core.CoreManager;
import org.correomqtt.di.Observes;
import org.correomqtt.core.importexport.messages.ExportMessageFailedEvent;
import org.correomqtt.core.importexport.messages.ExportMessageStartedEvent;
import org.correomqtt.core.importexport.messages.ExportMessageSuccessEvent;
import org.correomqtt.core.model.MessageType;
import org.correomqtt.core.plugin.MessageValidator;
import org.correomqtt.core.plugin.spi.MessageValidatorHook;
import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.gui.contextmenu.DetailContextMenu;
import org.correomqtt.gui.contextmenu.DetailContextMenuDelegate;
import org.correomqtt.gui.contextmenu.DetailContextMenuFactory;
import org.correomqtt.gui.controls.IconCheckMenuItem;
import org.correomqtt.gui.formats.Format;
import org.correomqtt.gui.menuitem.DetailViewManipulatorTaskMenuItem;
import org.correomqtt.gui.model.MessagePropertiesDTO;
import org.correomqtt.gui.model.Search;
import org.correomqtt.gui.model.WindowProperty;
import org.correomqtt.gui.model.WindowType;
import org.correomqtt.gui.plugin.DetailViewManipulatorTask;
import org.correomqtt.gui.plugin.GuiPluginManager;
import org.correomqtt.gui.plugin.spi.DetailViewHook;
import org.correomqtt.gui.plugin.spi.DetailViewManipulatorHook;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.transformer.MessageTransformer;
import org.correomqtt.gui.utils.AutoFormatPayload;
import org.correomqtt.gui.utils.FxThread;
import org.correomqtt.gui.utils.MessageUtils;
import org.correomqtt.gui.utils.WindowHelper;
import org.correomqtt.gui.views.LoaderResult;
import org.fxmisc.flowless.VirtualizedScrollPane;
import org.fxmisc.richtext.CodeArea;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.ResourceBundle;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import java.util.regex.PatternSyntaxException;

@DefaultBean
public class DetailViewController extends BaseConnectionController implements
        DetailContextMenuDelegate {

    private static final Logger LOGGER = LoggerFactory.getLogger(DetailViewController.class);

    private static ResourceBundle resources;
    private final BooleanProperty inlineViewProperty;
    private final MessageValidator messageValidator;
    private final GuiPluginManager guiPluginManager;
    private final AutoFormatPayload autoFormatPayload;
    private final DetailContextMenuFactory detailContextMenuFactory;
    private final MessageUtils messageUtils;
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
    private IconCheckMenuItem ignoreCaseMenuItem;
    @FXML
    private IconCheckMenuItem regexMenuItem;
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

    private MessagePropertiesDTO messageDTO;

    private DetailViewManipulatorTask lastManipulatorTask;



    @Inject
    public DetailViewController(CoreManager coreManager,
                         MessageValidator messageValidator,
                         GuiPluginManager guiPluginManager,
                         AutoFormatPayload autoFormatPayload,
                         ThemeManager themeManager,
                         DetailContextMenuFactory detailContextMenuFactory,
                         MessageUtils messageUtils,
                         @Assisted MessagePropertiesDTO messageDTO,
                         @Assisted String connectionId,
                         @Assisted DetailViewDelegate delegate,
                         @Assisted boolean isInlineView) {
        super(coreManager, themeManager, connectionId);
        this.messageValidator = messageValidator;
        this.guiPluginManager = guiPluginManager;
        this.autoFormatPayload = autoFormatPayload;
        this.detailContextMenuFactory = detailContextMenuFactory;
        this.messageUtils = messageUtils;
        this.delegate = delegate;
        inlineViewProperty = new SimpleBooleanProperty(isInlineView);
        this.messageDTO = messageDTO;
    }

    @FXML
    private void initialize() {
        detailViewVBox.getStyleClass().add(themeManager.getIconModeCssClass());
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

        DetailContextMenu contextMenu = detailContextMenuFactory.create(this);

        metaHolder.setOnContextMenuRequested(event -> {
            if (messageDTO != null) {
                contextMenu.setObject(messageDTO);
                contextMenu.show(metaHolder, event.getScreenX(), event.getScreenY());
            }
        });

        detailViewFormatToggleButton.setOnMouseClicked(mouseEvent -> {
            autoFormatPayload.autoFormatPayload(messageDTO.getPayload(), detailViewFormatToggleButton.isSelected(), getConnectionId(), codeArea);
            showSearchResult();
        });

        initializeManipulation();

        setMessage(messageDTO);
    }

    private void initializeManipulation() {
        manipulateSelectionButton.getItems().clear();
        lastManipulatorTask = null;
        manipulateSelectionButton.setText("Manipulate");

        List<DetailViewManipulatorTask> tasks = guiPluginManager.getDetailViewManipulatorTasks();
        tasks.forEach(p -> {
            DetailViewManipulatorTaskMenuItem menuItem = new DetailViewManipulatorTaskMenuItem(p);
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

        coreManager.getSettingsManager().getSettings().setUseIgnoreCase(ignoreCaseMenuItem.isSelected());
        coreManager.getSettingsManager().saveSettings();
        this.searchMenuButton.getItems().remove(ignoreCaseMenuItem);
        this.searchMenuButton.getItems().add(0, ignoreCaseMenuItem);
        currentSearchString = null;
        currentSearchResult = 0;
        showSearchResult();
    }

    private void showSearchResult() {

        currentSearchString = searchTextField.textProperty().get();

        results = new ArrayList<>();

        if (!currentSearchString.isEmpty()) {
            String codeAreaText = codeArea.getText();

            boolean ignoreCase = coreManager.getSettingsManager().getSettings().isUseIgnoreCase();
            boolean regex = coreManager.getSettingsManager().getSettings().isUseRegexForSearch();

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
    private void changeRegex() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Clicked on changeRegex: {}", getConnectionId());
        }

        coreManager.getSettingsManager().getSettings().setUseRegexForSearch(regexMenuItem.isSelected());
        coreManager.getSettingsManager().saveSettings();
        this.searchMenuButton.getItems().remove(regexMenuItem);
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

        ignoreCaseMenuItem.setSelected(coreManager.getSettingsManager().getSettings().isUseIgnoreCase());
        regexMenuItem.setSelected(coreManager.getSettingsManager().getSettings().isUseRegexForSearch());

        detailViewSaveButton.setOnMouseClicked(this::saveMessage);

        if (messageDTO.getPayload().isEmpty()) {
            clearPayload();
        } else {
            updatePayload(messageDTO.getPayload());
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

    private void executeOnOpenDetailViewExtensions() {
        detailViewNodeBox.getChildren().clear();
        coreManager.getPluginManager().getExtensions(DetailViewHook.class).forEach(p -> {
            HBox pluginBox = new HBox();
            pluginBox.setAlignment(Pos.CENTER_RIGHT);
            HBox.setHgrow(pluginBox, Priority.ALWAYS);
            detailViewNodeBox.getChildren().add(pluginBox);
            p.onOpenDetailView(MessageTransformer.propsToExtensionDTO(messageDTO), pluginBox);
        });
    }

    private void validateMessage(String topic, String payload) {
        detailViewValid.setVisible(false);
        detailViewValid.setManaged(false);
        detailViewInvalid.setVisible(false);
        detailViewInvalid.setManaged(false);

        MessageValidatorHook.Validation validation = messageValidator.validateMessage(topic, payload);
        if (validation != null) {
            updateValidatorLabel(detailViewValid, validation.isValid(), validation.getTooltip());
            updateValidatorLabel(detailViewInvalid, !validation.isValid(), validation.getTooltip());
        }
    }

    private void saveMessage(MouseEvent event) {

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Clicked on save message: {}", getConnectionId());
        }

        Stage stage = (Stage) detailViewVBox.getScene().getWindow();
        messageUtils.saveMessage(messageDTO, stage);
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

        Format format = autoFormatPayload.autoFormatPayload(payload, true, getConnectionId(), codeArea);
        detailViewFormatToggleButton.setSelected(format.isFormatable());
        detailViewFormatToggleButton.setDisable(!format.isFormatable());
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
        DetailViewManipulatorTask manipulatorTask = ((DetailViewManipulatorTaskMenuItem) actionEvent.getSource()).getTask();
        manipulateMessage(manipulatorTask);
        manipulateSelectionButton.setText(manipulatorTask.getName());
        this.lastManipulatorTask = manipulatorTask;
    }

    private void manipulateMessage(DetailViewManipulatorTask manipulatorTask) {
        detailViewRevertManipulationButton.setDisable(false);

        IndexRange range = getSelectionRange();

        byte[] selection = codeArea.getText(range).getBytes();
        for (DetailViewManipulatorHook hook : manipulatorTask.getHooks()) {
            selection = hook.manipulate(selection);
        }

        codeArea.replaceText(range, new String(selection));
        detailViewFormatToggleButton.setSelected(false);

        if (messageDTO != null) {
            validateMessage(messageDTO.getTopic(), codeArea.getText());
            autoFormatPayload.autoFormatPayload(codeArea.getText(), true, getConnectionId(), codeArea);
        }
    }

    private IndexRange getSelectionRange() {
        IndexRange selection = codeArea.getSelection();
        if (selection.getLength() == 0) {
            return new IndexRange(0, codeArea.getLength());
        }
        return selection;
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

    Node getMainNode() {
        return detailViewVBox;
    }

    @Override
    public void showDetailsInSeparateWindow(MessagePropertiesDTO messageDTO) {
        showAsDialog();
    }

    void showAsDialog() {

        Map<Object, Object> properties = new HashMap<>();
        properties.put(WindowProperty.WINDOW_TYPE, WindowType.DETAIL);
        properties.put(WindowProperty.CONNECTION_ID, connectionId);
        properties.put(WindowProperty.MESSAGE_ID, messageDTO.getMessageId());

        if (WindowHelper.focusWindowIfAlreadyThere(properties)) {
            return;
        }

        LoaderResult<DetailViewController> result = load();

        resources = result.getResourceBundle();

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

    LoaderResult<DetailViewController> load() {
        return load(DetailViewController.class, "detailView.fxml", () -> this);
    }

    @Override
    public void setUpToForm(MessagePropertiesDTO messageDTO) {
        delegate.setUpToForm(messageDTO);
    }

    @Override
    public BooleanProperty isInlineView() {
        return inlineViewProperty;
    }


    @FxThread
    @SuppressWarnings("unused")
    @Observes(ExportMessageStartedEvent.class)
    public void onExportStarted() {
        Platform.runLater(() -> detailViewVBox.setDisable(true));
    }

    @FxThread
    @SuppressWarnings("unused")
    @Observes(ExportMessageSuccessEvent.class)
    public void onExportSucceeded() {
        Platform.runLater(() -> detailViewVBox.setDisable(false));
    }

    @FxThread
    @SuppressWarnings("unused")
    @Observes(ExportMessageFailedEvent.class)
    public void onExportFailed() {
        Platform.runLater(() -> detailViewVBox.setDisable(false));
    }

    public void cleanUp() {
    }
}
