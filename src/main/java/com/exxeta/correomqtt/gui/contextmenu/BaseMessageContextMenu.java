package com.exxeta.correomqtt.gui.contextmenu;

import com.exxeta.correomqtt.business.services.ConfigService;
import com.exxeta.correomqtt.gui.model.MessagePropertiesDTO;
import com.exxeta.correomqtt.gui.helper.ClipboardHelper;
import javafx.event.ActionEvent;
import javafx.scene.control.MenuItem;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;

abstract class BaseMessageContextMenu<D extends BaseMessageContextMenuDelegate> extends BaseObjectContextMenu<MessagePropertiesDTO, D> {

    private static final Logger LOGGER = LoggerFactory.getLogger(BaseMessageContextMenu.class);

    protected MenuItem putToForm;
    protected MenuItem showDetails;
    protected MenuItem copyTopicToClipboard;
    protected MenuItem copyTimeToClipboard;
    protected MenuItem copyPayloadToClipboard;
    private ResourceBundle resources;

    BaseMessageContextMenu(D dispatcher) {
        super(dispatcher);
    }

    @Override
    protected void initializeItems() {

        resources = ResourceBundle.getBundle("com.exxeta.correomqtt.i18n", ConfigService.getInstance().getSettings().getCurrentLocale());
        super.initializeItems();

        putToForm = new MenuItem(resources.getString("baseMessageContextMenuPutToFormMenuItem"));
        putToForm.setOnAction(this::putToForm);

        showDetails = new MenuItem(resources.getString("baseMessageContextMenuShowDetailsMenuItem"));
        showDetails.setOnAction(this::showDetails);

        copyTopicToClipboard = new MenuItem(resources.getString("baseMessageContextMenuTopicMenuItem"));
        copyTopicToClipboard.setOnAction(this::copyTopicToClipboard);

        copyTimeToClipboard = new MenuItem(resources.getString("baseMessageContextMenuTimeMenuItem"));
        copyTimeToClipboard.setOnAction(this::copyTimeToClipboard);

        copyPayloadToClipboard = new MenuItem(resources.getString("baseMessageContextMenuPayloadMenuItem"));
        copyPayloadToClipboard.setOnAction(this::copyPayloadToClipoard);

    }

    @Override
    protected void setVisibilityForObjectItems(boolean visible) {
        super.setVisibilityForObjectItems(visible);
        putToForm.setVisible(visible);
        showDetails.setVisible(visible);
        copyTopicToClipboard.setVisible(visible);
        copyTimeToClipboard.setVisible(visible);
        copyPayloadToClipboard.setVisible(visible);
    }

    private void showDetails(@SuppressWarnings("unused") ActionEvent actionEvent) {
        if (dto != null) {
            delegate.showDetailsInSeparateWindow(dto);
        } else {
            LOGGER.warn("Call to {}::showDetails with empty message.", getClassName());
        }
    }

    private void copyPayloadToClipoard(@SuppressWarnings("unused") ActionEvent actionEvent) {
        if (dto != null) {
            ClipboardHelper.addToClipboard(dto.getPayload());
        } else {
            LOGGER.warn("Call to {}::copyPayloadToClipoard with empty message.", getClassName());
        }
    }

    private void copyTimeToClipboard(@SuppressWarnings("unused") ActionEvent actionEvent) {
        if (dto != null) {
            ClipboardHelper.addToClipboard(dto.getDateTime().toString());
        } else {
            LOGGER.warn("Call to {}::copyTimeToClipboard with empty message.", getClassName());
        }
    }

    private void copyTopicToClipboard(@SuppressWarnings("unused") ActionEvent actionEvent) {
        if (dto != null) {
            ClipboardHelper.addToClipboard(dto.getTopic());
        } else {
            LOGGER.warn("Call to {}::copyTopicToClipboard with empty message.", getClassName());
        }
    }

    private void putToForm(@SuppressWarnings("unused") ActionEvent actionEvent) {
        if (dto != null) {
            delegate.setUpToForm(dto);
        } else {
            LOGGER.warn("Call to {}::putToForm with empty message.", getClassName());
        }
    }

    public ResourceBundle getResources() {
        return resources;
    }

    public void setResources(ResourceBundle resources) {
        this.resources = resources;
    }
}
