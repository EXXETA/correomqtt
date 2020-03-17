package org.correomqtt.gui.contextmenu;

import com.exxeta.correomqtt.gui.model.MessagePropertiesDTO;
import javafx.event.ActionEvent;
import javafx.scene.control.MenuItem;
import javafx.scene.control.SeparatorMenuItem;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class MessageListContextMenu extends BaseMessageContextMenu<MessageListContextMenuDelegate> {

    private static final Logger LOGGER = LoggerFactory.getLogger(MessageListContextMenu.class);

    private MenuItem removeMessage;
    private MenuItem saveMessage;
    private MenuItem timeInfo;
    private MenuItem clearList;

    private SeparatorMenuItem separator1;
    private SeparatorMenuItem separator2;
    private SeparatorMenuItem separator3;

    public MessageListContextMenu(MessageListContextMenuDelegate dispatcher) {
        super(dispatcher);
    }

    @Override
    protected void initializeItems() {

        super.initializeItems();
        removeMessage = new MenuItem(getResources().getString("messageListContextMenuRemoveMenuItem"));
        removeMessage.setOnAction(this::removeMessage);

        saveMessage = new MenuItem(getResources().getString("messageListContextMenuSaveMenuItem"));
        saveMessage.setOnAction(this::saveMessage);

        timeInfo = new MenuItem();
        timeInfo.setVisible(false);
        timeInfo.setDisable(true);

        clearList = new MenuItem(getResources().getString("messageListContextMenuClearMenuItem"));
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
