package org.correomqtt.gui.views.scripting;

import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.collections.transformation.FilteredList;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.control.SplitPane;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Region;
import lombok.AllArgsConstructor;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.eventbus.Subscribe;
import org.correomqtt.core.eventbus.SubscribeFilter;
import org.correomqtt.core.eventbus.SubscribeFilterNames;
import org.correomqtt.core.scripting.BaseExecutionEvent;
import org.correomqtt.core.scripting.ExecutionDTO;
import org.correomqtt.core.scripting.ScriptDeleteExecutionsTask;
import org.correomqtt.core.scripting.ScriptExecutionCancelledEvent;
import org.correomqtt.core.scripting.ScriptExecutionFailedEvent;
import org.correomqtt.core.scripting.ScriptExecutionProgressEvent;
import org.correomqtt.core.scripting.ScriptExecutionSuccessEvent;
import org.correomqtt.core.scripting.ScriptExecutionTask;
import org.correomqtt.core.scripting.ScriptExecutionsDeletedEvent;
import org.correomqtt.core.scripting.ScriptingBackend;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;

import java.text.MessageFormat;
import java.util.HashMap;
import java.util.Map;
import java.util.ResourceBundle;

public class ExecutionViewController extends BaseControllerImpl {

    private static ResourceBundle resources;
    private final Map<String, ScriptExecutionState> executionStates = new HashMap<>();
    @FXML
    private AnchorPane executionSidebar;
    @FXML
    private SplitPane splitPane;
    @FXML
    private AnchorPane emptyExecution;
    @FXML
    private Label headerLabel;
    @FXML
    private Label emptyLabel;
    @FXML
    private AnchorPane executionHolder;
    @FXML
    private ListView<ExecutionPropertiesDTO> executionListView;
    private ObservableList<ExecutionPropertiesDTO> executionList;
    private FilteredList<ExecutionPropertiesDTO> filteredList;
    private String currentName;

    public static LoaderResult<ExecutionViewController> load() {

        LoaderResult<ExecutionViewController> result = load(ExecutionViewController.class, "executionView.fxml",
                ExecutionViewController::new);
        resources = result.getResourceBundle();
        return result;
    }

    public ExecutionViewController() {
        EventBus.register(this);
    }

    public void renameScript(String oldName, String newName) {
        executionList.stream()
                .filter(e -> e.getScriptFilePropertiesDTO().getName().equals(oldName))
                .forEach(e -> e.getScriptFilePropertiesDTO().getNameProperty().set(newName));
    }

    public void onClearExecutionsClicked() {
        new ScriptDeleteExecutionsTask(currentName)
                .onError(error -> AlertHelper.unexpectedAlert(error.getUnexpectedError()))
                .run();

    }

    @SuppressWarnings("unused")
    @Subscribe(ScriptExecutionsDeletedEvent.class)
    public void onExecutionsDeleted() {
        executionList.clear();
    }

    @SubscribeFilter(SubscribeFilterNames.SCRIPT_NAME)
    public String getScriptFileName() {
        return currentName;
    }

    public void cleanup() {
        for (ScriptExecutionState state : executionStates.values()) {
            state.controller.cleanup();
        }
        EventBus.unregister(this);
    }

    public void filterByScript(String name) {
        currentName = name;
        filteredList.setPredicate(s -> s.getScriptFilePropertiesDTO().getName().equals(name));
        executionListView.getSelectionModel().selectFirst();
        headerLabel.setText(MessageFormat.format(resources.getString("scripting.executions"), name));
        updateExistence();
    }

    private void updateExistence() {
        if (executionListView.getItems().isEmpty()) {
            emptyLabel.setText(MessageFormat.format(resources.getString("emptyScriptExecutionArea"), currentName));
            if (!splitPane.getItems().contains(emptyExecution)) {
                splitPane.getItems().add(emptyExecution);
            }
            splitPane.getItems().remove(executionSidebar);
            splitPane.getItems().remove(executionHolder);
        } else {

            splitPane.setDividerPositions(0.3, 0.7); //TODO persist and remember user choice instead of forcing

            splitPane.getItems().remove(emptyExecution);
            if (!splitPane.getItems().contains(executionSidebar)) {
                splitPane.getItems().add(executionSidebar);
            }

            if (!splitPane.getItems().contains(executionHolder)) {
                splitPane.getItems().add(executionHolder);
            }
        }
    }

    @FXML
    private void initialize() {

        executionList = FXCollections.observableArrayList(ScriptingBackend.getExecutions()
                .stream()
                .map(ExecutionTransformer::dtoToProps)
                .toList());
        filteredList = new FilteredList<>(executionList, e -> false);

        executionListView.setItems(filteredList);

        executionListView.setCellFactory(this::createExcecutionCell);
        executionListView.getSelectionModel().selectFirst();

    }

    private ListCell<ExecutionPropertiesDTO> createExcecutionCell(ListView<ExecutionPropertiesDTO> executionListView) {
        ExecutionCell cell = new ExecutionCell(this.executionListView);
        cell.selectedProperty().addListener((observable, oldValue, newValue) -> {
            if (Boolean.TRUE.equals(newValue)) {
                onSelectExecution(cell.getItem());
            } else {
                if (executionListView.getSelectionModel().getSelectedIndices().isEmpty()) {
                    clearExecution();
                }
            }
        });
        return cell;
    }

    private void onSelectExecution(ExecutionPropertiesDTO selectedItem) {
        executionHolder.getChildren().clear();
        executionHolder.getChildren().add(getExecutionState(selectedItem).region);
    }

    private void clearExecution() {
        executionHolder.getChildren().clear();
    }

    private ScriptExecutionState getExecutionState(ExecutionPropertiesDTO dto) {
        return executionStates.computeIfAbsent(dto.getExecutionId(),
                id -> {
                    LoaderResult<SingleExecutionViewController> loaderResult = SingleExecutionViewController.load(dto);
                    return new ScriptExecutionState(loaderResult.getController(), loaderResult.getMainRegion());
                });
    }

    public boolean addExecution(ScriptFilePropertiesDTO dto, ConnectionPropertiesDTO selectedConnection, String jsCode) {

        if (selectedConnection == null) {
            AlertHelper.warn(resources.getString("scriptStartWithoutConnectionNotPossibleTitle"),
                    resources.getString("scriptStartWithoutConnectionNotPossibleContent"));
            return false;
        }

        if (dto == null) {
            AlertHelper.warn(resources.getString("scriptStartWithoutConnectionNotPossibleTitle"), // TODO custom error here
                    resources.getString("scriptStartWithoutConnectionNotPossibleContent"));
            return false;
        }

        ExecutionDTO executionDTO = ExecutionDTO.builder()
                .jsCode(jsCode)
                .scriptFile(ScriptingTransformer.propsToDTO(dto))
                .connectionId(selectedConnection.getId())
                .build();

        ExecutionPropertiesDTO executionPropertyDTO = ExecutionTransformer.dtoToProps(executionDTO);
        executionList.add(0, executionPropertyDTO);
        executionListView.getSelectionModel().selectFirst();
        updateExistence();

        // Events used here to keep state in background even if window is closed in the meantime.
        new ScriptExecutionTask(executionDTO)
                .onError(error -> AlertHelper.unexpectedAlert(error.getUnexpectedError()))
                .run();
        return true;
    }

    @SuppressWarnings("unused")
    public void onScriptExecutionCancelled(@Subscribe ScriptExecutionCancelledEvent event) {
        handleScriptExecutionResult(event);
    }

    private void handleScriptExecutionResult(BaseExecutionEvent event) {
        ExecutionDTO dto = event.getExecutionDTO();
        if (dto == null)
            return;
        ExecutionPropertiesDTO props = executionList.stream()
                .filter(epd -> epd.getExecutionId().equals(dto.getExecutionId()))
                .findFirst()
                .orElseThrow();
        ExecutionTransformer.updatePropsByDto(props, dto);
    }

    @SuppressWarnings("unused")
    public void onScriptExecutionSuccess(@Subscribe ScriptExecutionSuccessEvent event) {
        handleScriptExecutionResult(event);
    }

    @SuppressWarnings("unused")
    public void onScriptExecutionFailed(@Subscribe ScriptExecutionFailedEvent event) {
        handleScriptExecutionResult(event);
    }

    @SuppressWarnings("unused")
    public void onScriptExecutionProgress(@Subscribe ScriptExecutionProgressEvent event) {
        handleScriptExecutionResult(event);
    }

    @AllArgsConstructor
    private static class ScriptExecutionState {
        private SingleExecutionViewController controller;
        private Region region;
    }
}