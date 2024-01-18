package org.correomqtt.gui.contextmenu;

import javafx.event.ActionEvent;
import org.correomqtt.business.fileprovider.SettingsProvider;
import org.correomqtt.gui.controls.IconMenuItem;
import org.correomqtt.gui.model.MessagePropertiesDTO;
import org.correomqtt.gui.utils.ClipboardHelper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;

@SuppressWarnings("java:S110")
abstract class BaseMessageContextMenu<D extends BaseMessageContextMenuDelegate> extends BaseObjectContextMenu<MessagePropertiesDTO, D> {

    private static final Logger LOGGER = LoggerFactory.getLogger(BaseMessageContextMenu.class);

    protected IconMenuItem putToForm;
    protected IconMenuItem showDetails;
    protected IconMenuItem copyTopicToClipboard;
    protected IconMenuItem copyTimeToClipboard;
    protected IconMenuItem copyPayloadToClipboard;
    private ResourceBundle resources;

    BaseMessageContextMenu(D dispatcher) {
        super(dispatcher);
    }

    @Override
    protected void initializeItems() {

        resources = ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale());
        super.initializeItems();

        putToForm = new IconMenuItem(resources.getString("baseMessageContextMenuPutToFormMenuItem"));
        putToForm.setIcon("mdi-arrow-expand-up");
        putToForm.setOnAction(this::putToForm);

        showDetails = new IconMenuItem(resources.getString("baseMessageContextMenuShowDetailsMenuItem"));
        showDetails.setIcon("mdi-open-in-new");
        showDetails.setOnAction(this::showDetails);

        copyTopicToClipboard = new IconMenuItem(resources.getString("baseMessageContextMenuTopicMenuItem"));
        copyTopicToClipboard.setIcon("mdi-clipboard");
        copyTopicToClipboard.setOnAction(this::copyTopicToClipboard);

        copyTimeToClipboard = new IconMenuItem(resources.getString("baseMessageContextMenuTimeMenuItem"));
        copyTimeToClipboard.setIcon("mdi-clipboard-clock");
        copyTimeToClipboard.setOnAction(this::copyTimeToClipboard);

        copyPayloadToClipboard = new IconMenuItem(resources.getString("baseMessageContextMenuPayloadMenuItem"));
        copyPayloadToClipboard.setIcon("mdi-clipboard-text");
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
