use correo_core::{AppCommand, AppCommandSender, AppSnapshot, SubscriptionRow};
use correo_style::layout;
use egui::{Button, Rect, RichText, Sense, Ui};
use egui_phosphor::regular;

use crate::{
    responsive,
    theme::{ThemeTokens, CONTROL_HEIGHT},
    widgets::{
        edit_pulldown, fill_remaining_tile_rows, square_icon_button_size,
        tile_scroll_bar_rect_with_height, tile_table_fill, with_icon_button_padding,
    },
    workbench_connection_messages::{self, MessageOrigin},
    workbench_helpers::{
        child_ui, connected, disconnected_action_button, qos_selector, right_rect, send,
        toolbar_rect,
    },
    workbench_layout::{self, WorkbenchPaneSide},
};

pub(crate) fn editor(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    workbench_layout::pane_title(ui, "Subscribe", WorkbenchPaneSide::Subscribe);
    ui.add_space(4.0);
    topic_row(ui, snapshot, tokens, commands);
    subscriptions(ui, snapshot, tokens, commands);
}

pub(crate) fn incoming_messages(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    workbench_connection_messages::show(ui, snapshot, MessageOrigin::Incoming, tokens, commands);
}

fn topic_row(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let mut topic = snapshot.workbench.subscribe.topic.clone();
    let rect = toolbar_rect(ui);
    let icon_only = responsive::workbench_uses_icon_actions(rect.width());
    let is_connected = connected(snapshot);
    let can_subscribe = snapshot.workbench.subscribe.valid && is_connected;
    let action_width = if icon_only {
        square_icon_button_size()[0]
    } else {
        layout::SUBSCRIBE_ACTION_BUTTON_WIDTH
    };

    let subscribe_rect = right_rect(rect, action_width, 0.0);
    let qos_rect = right_rect(rect, layout::QOS_WIDTH, action_width + layout::TOOLBAR_GAP);
    let topic_rect = Rect::from_min_max(
        rect.left_top(),
        egui::pos2(
            (qos_rect.left() - layout::TOOLBAR_GAP).max(rect.left()),
            rect.bottom(),
        ),
    );

    child_ui(ui, topic_rect, |ui| {
        let topic_response = edit_pulldown(
            ui,
            "subscribe-topic",
            &mut topic,
            "Topic filter",
            &snapshot.workbench.subscribe.topic_history,
            topic_rect.width(),
        );
        if topic_response.changed() {
            send(commands, AppCommand::UpdateSubscribeTopic(topic));
        }
    });
    child_ui(ui, qos_rect, |ui| {
        qos_selector(
            ui,
            "subscribe-qos",
            snapshot.workbench.subscribe.qos,
            |qos| {
                send(commands, AppCommand::UpdateSubscribeQos(qos));
            },
        );
    });
    child_ui(ui, subscribe_rect, |ui| {
        let label = if icon_only {
            regular::ARROW_DOWN_LEFT.to_owned()
        } else {
            format!("{}  Subscribe", regular::ARROW_DOWN_LEFT)
        };
        if !is_connected {
            disconnected_action_button(
                ui,
                subscribe_rect.width(),
                label,
                "Subscribe is not available as long as the connection is not connected.",
                tokens,
            );
            return;
        }

        let subscribe = ui.add_enabled_ui(can_subscribe, |ui| {
            ui.spacing_mut().button_padding.x = 4.0;
            ui.add_sized(
                [subscribe_rect.width(), CONTROL_HEIGHT],
                Button::new(&label),
            )
        });
        let subscribe = subscribe.inner;
        if subscribe.clicked() {
            send(commands, AppCommand::Subscribe);
        }
        if !can_subscribe {
            subscribe.on_hover_text("Requires a valid topic filter.");
        } else if icon_only {
            subscribe.on_hover_text("Subscribe");
        }
    });
}

fn subscriptions(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    subscription_toolbar(ui, snapshot, commands);
    subscription_table(ui, snapshot, tokens, commands);
}

fn subscription_toolbar(ui: &mut Ui, snapshot: &AppSnapshot, commands: &AppCommandSender) {
    ui.horizontal(|ui| {
        if ui
            .add_enabled(
                selected_subscription_count(snapshot) > 0,
                Button::new("Unsubscribe"),
            )
            .clicked()
        {
            for subscription in snapshot
                .workbench
                .subscribe
                .subscriptions
                .iter()
                .filter(|subscription| subscription.selected)
            {
                send(
                    commands,
                    AppCommand::Unsubscribe(subscription.topic_filter.clone()),
                );
            }
        }
        if ui
            .add_enabled(
                snapshot.workbench.subscribe.subscriptions.len() > 1,
                Button::new("Unsubscribe All"),
            )
            .clicked()
        {
            send(commands, AppCommand::UnsubscribeAll);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_enabled_ui(true, |ui| {
                    with_icon_button_padding(ui, |ui| {
                        ui.add_sized(
                            square_icon_button_size(),
                            Button::new(RichText::new(regular::FUNNEL_X).size(16.0)),
                        )
                    })
                })
                .inner
                .on_hover_text("Select none")
                .clicked()
            {
                send(
                    commands,
                    AppCommand::SetAllSubscriptionMessagesVisible(false),
                );
            }
            if ui
                .add_enabled_ui(true, |ui| {
                    with_icon_button_padding(ui, |ui| {
                        ui.add_sized(
                            square_icon_button_size(),
                            Button::new(RichText::new(regular::FUNNEL).size(16.0)),
                        )
                    })
                })
                .inner
                .on_hover_text("Select all")
                .clicked()
            {
                send(
                    commands,
                    AppCommand::SetAllSubscriptionMessagesVisible(true),
                );
            }
        });
    });
}

fn subscription_table(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.spacing_mut().item_spacing.y = 0.0;
    let table_height =
        (ui.available_height() - layout::TABLE_SCROLL_BOTTOM_GAP).max(layout::TABLE_MIN_HEIGHT);
    egui::ScrollArea::vertical()
        .id_salt("subscriptions-table")
        .max_height(table_height)
        .scroll_bar_rect(tile_scroll_bar_rect_with_height(ui, table_height))
        .auto_shrink([false, false])
        .show_rows(
            ui,
            layout::SUBSCRIPTION_ROW_HEIGHT,
            snapshot.workbench.subscribe.subscriptions.len(),
            |ui, row_range| {
                let subscriptions = &snapshot.workbench.subscribe.subscriptions;
                for index in row_range {
                    if let Some(subscription) = subscriptions.get(index) {
                        subscription_row(ui, index, subscription, tokens, commands);
                    }
                }
            },
        );
    fill_remaining_tile_rows(
        ui,
        snapshot.workbench.subscribe.subscriptions.len(),
        layout::SUBSCRIPTION_ROW_HEIGHT,
        table_height,
        tokens,
    );
}

fn subscription_row(
    ui: &mut Ui,
    index: usize,
    subscription: &SubscriptionRow,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let rect = ui.available_rect_before_wrap();
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(rect.width(), layout::SUBSCRIPTION_ROW_HEIGHT),
        Sense::click(),
    );
    let paint_rect = rect;

    let fill = if subscription.selected {
        ui.visuals().selection.bg_fill
    } else if response.hovered() {
        ui.visuals().widgets.hovered.bg_fill
    } else {
        tile_table_fill(index, tokens)
    };
    ui.painter()
        .rect_filled(paint_rect, egui::CornerRadius::ZERO, fill);

    let toggle_width = layout::SUBSCRIPTION_TOGGLE_WIDTH;
    let pill_width = layout::SUBSCRIPTION_QOS_SLOT_WIDTH;
    let toggle_rect = Rect::from_min_max(
        egui::pos2(
            paint_rect.right() - toggle_width - layout::SUBSCRIPTION_ROW_PADDING_RIGHT,
            paint_rect.top(),
        ),
        egui::pos2(
            paint_rect.right() - layout::SUBSCRIPTION_ROW_PADDING_RIGHT,
            paint_rect.bottom(),
        ),
    );
    let pill_rect = Rect::from_min_max(
        egui::pos2(toggle_rect.left() - pill_width, paint_rect.top()),
        egui::pos2(toggle_rect.left(), paint_rect.bottom()),
    );
    let topic_rect = Rect::from_min_max(
        egui::pos2(
            paint_rect.left() + layout::SUBSCRIPTION_ROW_PADDING_X,
            paint_rect.top(),
        ),
        egui::pos2(pill_rect.left() - layout::TOOLBAR_GAP, paint_rect.bottom()),
    );

    let text_pos = egui::pos2(
        topic_rect.left(),
        topic_rect.center().y - ui.text_style_height(&egui::TextStyle::Button) * 0.5,
    );
    ui.painter().text(
        text_pos,
        egui::Align2::LEFT_TOP,
        &subscription.topic_filter,
        egui::TextStyle::Button.resolve(ui.style()),
        ui.visuals().text_color(),
    );

    let pill = Rect::from_center_size(
        pill_rect.center(),
        egui::vec2(
            layout::SUBSCRIPTION_QOS_PILL_WIDTH,
            layout::SUBSCRIPTION_QOS_PILL_HEIGHT,
        ),
    );
    ui.painter().rect_filled(
        pill,
        egui::CornerRadius::same(99),
        ui.visuals().widgets.inactive.bg_fill.gamma_multiply(1.7),
    );
    ui.painter().text(
        pill.center(),
        egui::Align2::CENTER_CENTER,
        subscription.qos.label(),
        egui::TextStyle::Small.resolve(ui.style()),
        ui.visuals().text_color(),
    );

    let button_side = layout::square_icon_button_side();
    let button_size = egui::vec2(button_side, button_side);
    let button_rect = Rect::from_center_size(toggle_rect.center(), button_size);
    let toggle_response = ui
        .interact(
            button_rect,
            ui.id()
                .with(("subscription-filter-toggle", &subscription.topic_filter)),
            Sense::click(),
        )
        .on_hover_text("Show messages for this subscription");
    if response.clicked() && !toggle_response.clicked() {
        let modifiers = ui.input(|input| input.modifiers);
        send(
            commands,
            AppCommand::SelectSubscription {
                topic_filter: subscription.topic_filter.clone(),
                extend: modifiers.shift,
                toggle: modifiers.command || modifiers.ctrl || subscription.selected,
            },
        );
    }
    let base_fill = tokens.panel_raised;
    let toggle_fill = if toggle_response.hovered() || toggle_response.is_pointer_button_down_on() {
        ui.visuals().widgets.hovered.bg_fill
    } else if subscription.messages_visible {
        base_fill.gamma_multiply(1.85)
    } else {
        base_fill.gamma_multiply(1.15)
    };
    ui.painter().rect_filled(
        button_rect,
        ui.visuals().widgets.inactive.corner_radius,
        toggle_fill,
    );
    ui.painter().text(
        button_rect.center(),
        egui::Align2::CENTER_CENTER,
        if subscription.messages_visible {
            regular::FUNNEL
        } else {
            regular::FUNNEL_X
        },
        egui::FontId::proportional(15.0),
        ui.visuals().text_color(),
    );
    if toggle_response.clicked() {
        send(
            commands,
            AppCommand::SetSubscriptionMessagesVisible {
                topic_filter: subscription.topic_filter.clone(),
                visible: !subscription.messages_visible,
            },
        );
    }
}

fn selected_subscription_count(snapshot: &AppSnapshot) -> usize {
    snapshot
        .workbench
        .subscribe
        .subscriptions
        .iter()
        .filter(|subscription| subscription.selected)
        .count()
}
