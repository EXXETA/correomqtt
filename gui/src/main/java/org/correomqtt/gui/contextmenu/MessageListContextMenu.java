package org.correomqtt.gui.contextmenu;

import javafx.event.ActionEvent;
import javafx.scene.control.SeparatorMenuItem;
import org.correomqtt.gui.controls.IconMenuItem;
import org.correomqtt.gui.model.MessagePropertiesDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@SuppressWarnings("java:S110")
public class MessageListContextMenu extends BaseMessageContextMenu<MessageListContextMenuDelegate> {

    private static final Logger LOGGER = LoggerFactory.getLogger(MessageListContextMenu.class);

    private IconMenuItem removeMessage;
    private IconMenuItem saveMessage;
    private IconMenuItem timeInfo;

    private SeparatorMenuItem separator1;
    private SeparatorMenuItem separator2;
    private SeparatorMenuItem separator3;

    public MessageListContextMenu(MessageListContextMenuDelegate dispatcher) {
        super(dispatcher);
    }

    @Override
    protected void initializeItems() {

        super.initializeItems();
        removeMessage = new IconMenuItem(getResources().getString("messageListContextMenuRemoveMenuItem"));
        removeMessage.setIcon("mdi-trash-can");
        removeMessage.setOnAction(this::removeMessage);

        saveMessage = new IconMenuItem(getResources().getString("messageListContextMenuSaveMenuItem"));
        saveMessage.setIcon("mdi-content-save");
        saveMessage.setOnAction(this::saveMessage);

        timeInfo = new IconMenuItem();
        timeInfo.setVisible(false);
        timeInfo.setDisable(true);

        IconMenuItem clearList = new IconMenuItem(getResources().getString("messageListContextMenuClearMenuItem"));
        clearList.setIcon("mdi-notification-clear-all");
        clearList.setOnAction(this::clearList);

        separator1 = new SeparatorMenuItem();
        separator2 = new SeparatorMenuItem();
        separator3 = new SeparatorMenuItem();

        this.getItems().addAll(putToForm,
                               showDetails,
                               removeMessage,
                               saveMessage,
                               separator1,
                               copyTopicToClipboard,
                               copyTimeToClipboard,
                               copyPayloadToClipboard,
                               separator2,
                               timeInfo,
                               separator3,
                clearList);

        updateDateTime();
    }

    private void saveMessage(ActionEvent actionEvent) {
        if (dto != null) {
            delegate.saveMessage(dto);
        } else {
            LOGGER.warn("Call to {}::saveMessage with empty message.", getClassName());
        }
    }


    @Override
    public void setObject(MessagePropertiesDTO messageDTO){
        super.setObject(messageDTO);
        if(messageDTO != null) {
            messageDTO.getDateTimeProperty().addListener((observable, oldValue, newValue) -> updateDateTime());
        }
        updateDateTime();
    }

    @Override
    protected void setVisibilityForObjectItems(boolean visible) {
        super.setVisibilityForObjectItems(visible);
        removeMessage.setVisible(visible);
        saveMessage.setVisible(visible);
        separator1.setVisible(visible);
        separator2.setVisible(visible);
        separator3.setVisible(visible);
    }

    private void updateDateTime() {
        if (dto != null && dto.getDateTime() != null) {
            timeInfo.setVisible(true);
            timeInfo.setText(dto.getDateTime().toString()); //todo format
        } else {
            timeInfo.setVisible(false);
            timeInfo.setText("");
        }
    }

    private void clearList(@SuppressWarnings("unused") ActionEvent actionEvent) {
        delegate.clearList();
    }

    private void removeMessage(@SuppressWarnings("unused") ActionEvent actionEvent) {
        if (dto != null) {
            delegate.removeMessage(dto);
        } else {
            LOGGER.warn("Call to {}::removeMessage with empty message.", getClassName());
        }
    }

}
