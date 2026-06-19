use correo_core::{AppCommand, AppCommandSender, AppSnapshot, SubscriptionRow};
use correo_style::layout;
use egui::{Button, Id, Rect, RichText, Sense, Ui};
use egui_phosphor::regular;

use crate::{
    responsive,
    theme::{ThemeTokens, CONTROL_HEIGHT},
    widgets::{
        dotted_focus_outline, edit_pulldown, fill_remaining_tile_rows, paint_focus_outline,
        square_icon_button_size, tile_scroll_bar_rect_with_height, tile_table_hover_fill,
        tile_table_interactive_fill, with_icon_button_padding,
    },
    workbench_connection_messages::{self, MessageOrigin},
    workbench_helpers::{
        child_ui, connected, disconnected_action_button, qos_selector, right_rect, send,
        toolbar_rect,
    },
    workbench_layout::{self, WorkbenchPaneSide},
};

const SUBSCRIPTION_TABLE_HEIGHT_ADJUST: f32 = 4.0;

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
        paint_focus_outline(ui, &subscribe);
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
        let unsubscribe = ui.add_enabled(
            selected_subscription_count(snapshot) > 0,
            Button::new("Unsubscribe"),
        );
        paint_focus_outline(ui, &unsubscribe);
        if unsubscribe.clicked() {
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
        let unsubscribe_all = ui.add_enabled(
            snapshot.workbench.subscribe.subscriptions.len() > 1,
            Button::new("Unsubscribe All"),
        );
        paint_focus_outline(ui, &unsubscribe_all);
        if unsubscribe_all.clicked() {
            send(commands, AppCommand::UnsubscribeAll);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let select_none = ui
                .add_enabled_ui(true, |ui| {
                    with_icon_button_padding(ui, |ui| {
                        ui.add_sized(
                            square_icon_button_size(),
                            Button::new(RichText::new(regular::LIST_DASHES).size(16.0)),
                        )
                    })
                })
                .inner
                .on_hover_text("Select none");
            paint_focus_outline(ui, &select_none);
            if select_none.clicked() {
                send(
                    commands,
                    AppCommand::SetAllSubscriptionMessagesVisible(false),
                );
            }
            let select_all = ui
                .add_enabled_ui(true, |ui| {
                    with_icon_button_padding(ui, |ui| {
                        ui.add_sized(
                            square_icon_button_size(),
                            Button::new(RichText::new(regular::LIST_CHECKS).size(16.0)),
                        )
                    })
                })
                .inner
                .on_hover_text("Select all");
            paint_focus_outline(ui, &select_all);
            if select_all.clicked() {
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
    ui.add_space(3.0);
    let table_height = (ui.available_height() - layout::TABLE_SCROLL_BOTTOM_GAP
        + SUBSCRIPTION_TABLE_HEIGHT_ADJUST)
        .max(layout::TABLE_MIN_HEIGHT);
    let subscriptions = &snapshot.workbench.subscribe.subscriptions;
    let table_id = subscription_table_focus_id();
    let table_rect = egui::Rect::from_min_size(
        ui.available_rect_before_wrap().min,
        egui::vec2(ui.available_width(), table_height),
    );
    let table_response = ui.interact(table_rect, table_id, Sense::focusable_noninteractive());
    if table_response.has_focus() {
        ui.memory_mut(|memory| {
            memory.set_focus_lock_filter(
                table_id,
                egui::EventFilter {
                    vertical_arrows: true,
                    ..Default::default()
                },
            );
        });
        handle_subscription_table_keyboard(ui, subscriptions, commands);
    }
    if table_response.gained_focus() {
        store_subscription_focus_index(ui, selected_subscription_index(subscriptions).unwrap_or(0));
    }
    let focused_index = subscription_focus_index(ui, subscriptions);
    egui::ScrollArea::vertical()
        .id_salt("subscriptions-table")
        .max_height(table_height)
        .scroll_bar_rect(tile_scroll_bar_rect_with_height(ui, table_height))
        .auto_shrink([false, false])
        .show_rows(
            ui,
            layout::SUBSCRIPTION_ROW_HEIGHT,
            subscriptions.len(),
            |ui, row_range| {
                for index in row_range {
                    if let Some(subscription) = subscriptions.get(index) {
                        subscription_row(
                            ui,
                            index,
                            subscription,
                            tokens,
                            commands,
                            table_response.has_focus(),
                            focused_index,
                        );
                    }
                }
                fill_remaining_tile_rows(
                    ui,
                    subscriptions.len(),
                    layout::SUBSCRIPTION_ROW_HEIGHT,
                    table_height,
                    tokens,
                );
            },
        );
}

fn subscription_row(
    ui: &mut Ui,
    index: usize,
    subscription: &SubscriptionRow,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    table_focused: bool,
    focused_index: usize,
) {
    let rect = ui.available_rect_before_wrap();
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(rect.width(), layout::SUBSCRIPTION_ROW_HEIGHT),
        Sense::CLICK,
    );
    let paint_rect = rect;
    let row_focused = table_focused && index == focused_index;

    let fill = tile_table_interactive_fill(
        index,
        tokens,
        response.hovered() || row_focused,
        subscription.selected,
    );
    ui.painter()
        .rect_filled(paint_rect, egui::CornerRadius::ZERO, fill);
    if row_focused {
        dotted_focus_outline(ui, paint_rect);
    }

    let pill_width = layout::SUBSCRIPTION_QOS_SLOT_WIDTH;
    let checkbox_side = layout::square_icon_button_side();
    let checkbox_rect = Rect::from_center_size(
        egui::pos2(
            paint_rect.left() + layout::SUBSCRIPTION_ROW_PADDING_X + checkbox_side * 0.5,
            paint_rect.center().y,
        ),
        egui::vec2(checkbox_side, checkbox_side),
    );
    let pill_rect = Rect::from_min_max(
        egui::pos2(
            paint_rect.right() - pill_width - layout::SUBSCRIPTION_ROW_PADDING_RIGHT,
            paint_rect.top(),
        ),
        egui::pos2(
            paint_rect.right() - layout::SUBSCRIPTION_ROW_PADDING_RIGHT,
            paint_rect.bottom(),
        ),
    );
    let topic_rect = Rect::from_min_max(
        egui::pos2(
            checkbox_rect.right() + layout::TOOLBAR_GAP,
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
    let active_row = subscription.selected || response.hovered();
    let pill_bg = if active_row {
        brighter_color(tile_table_hover_fill(tokens), 1.2)
    } else {
        ui.visuals().widgets.inactive.bg_fill.gamma_multiply(1.7)
    };
    let pill_text = ui.visuals().text_color();
    ui.painter()
        .rect_filled(pill, egui::CornerRadius::same(99), pill_bg);
    ui.painter().text(
        pill.center(),
        egui::Align2::CENTER_CENTER,
        subscription.qos.label(),
        egui::TextStyle::Small.resolve(ui.style()),
        pill_text,
    );

    let checkbox_response = ui
        .interact(
            checkbox_rect,
            ui.make_persistent_id(("subscription-visible", &subscription.topic_filter)),
            Sense::CLICK,
        )
        .on_hover_text("Show messages for this subscription");
    if checkbox_response.hovered() || checkbox_response.is_pointer_button_down_on() {
        let visuals = ui.style().interact(&checkbox_response);
        ui.painter().rect(
            checkbox_rect,
            visuals.corner_radius,
            visuals.bg_fill,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );
    }
    ui.painter().text(
        checkbox_rect.center(),
        egui::Align2::CENTER_CENTER,
        if subscription.messages_visible {
            regular::CHECK_SQUARE
        } else {
            regular::SQUARE
        },
        egui::TextStyle::Button.resolve(ui.style()),
        ui.visuals().weak_text_color(),
    );
    if checkbox_response.clicked() {
        send(
            commands,
            AppCommand::SetSubscriptionMessagesVisible {
                topic_filter: subscription.topic_filter.clone(),
                visible: !subscription.messages_visible,
            },
        );
    }

    if response.clicked() && !checkbox_response.clicked() {
        ui.memory_mut(|memory| memory.request_focus(subscription_table_focus_id()));
        store_subscription_focus_index(ui, index);
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
}

fn handle_subscription_table_keyboard(
    ui: &Ui,
    subscriptions: &[SubscriptionRow],
    commands: &AppCommandSender,
) {
    if subscriptions.is_empty() {
        return;
    }
    let current = subscription_focus_index(ui, subscriptions);
    let next = ui.input(|input| {
        if input.key_pressed(egui::Key::ArrowDown) {
            Some((current + 1).min(subscriptions.len() - 1))
        } else if input.key_pressed(egui::Key::ArrowUp) {
            Some(current.saturating_sub(1))
        } else {
            None
        }
    });
    if let Some(next) = next {
        store_subscription_focus_index(ui, next);
        return;
    }
    if ui.input(|input| input.key_pressed(egui::Key::Enter)) {
        if let Some(subscription) = subscriptions.get(current) {
            send(
                commands,
                AppCommand::SelectSubscription {
                    topic_filter: subscription.topic_filter.clone(),
                    extend: false,
                    toggle: false,
                },
            );
        }
    } else if ui.input(|input| input.key_pressed(egui::Key::Space)) {
        if let Some(subscription) = subscriptions.get(current) {
            send(
                commands,
                AppCommand::SelectSubscription {
                    topic_filter: subscription.topic_filter.clone(),
                    extend: false,
                    toggle: true,
                },
            );
        }
    }
}

fn subscription_focus_index(ui: &Ui, subscriptions: &[SubscriptionRow]) -> usize {
    let selected = selected_subscription_index(subscriptions).unwrap_or(0);
    ui.ctx().data_mut(|data| {
        data.get_temp::<usize>(subscription_focus_index_id())
            .unwrap_or(selected)
            .min(subscriptions.len().saturating_sub(1))
    })
}

fn store_subscription_focus_index(ui: &Ui, index: usize) {
    ui.ctx()
        .data_mut(|data| data.insert_temp(subscription_focus_index_id(), index));
}

fn selected_subscription_index(subscriptions: &[SubscriptionRow]) -> Option<usize> {
    subscriptions
        .iter()
        .position(|subscription| subscription.selected)
}

fn subscription_table_focus_id() -> Id {
    Id::new("subscriptions-table-focus")
}

fn subscription_focus_index_id() -> Id {
    Id::new("subscriptions-table-focused-row")
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

fn brighter_color(color: egui::Color32, factor: f32) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        (color.r() as f32 * factor).round().min(255.0) as u8,
        (color.g() as f32 * factor).round().min(255.0) as u8,
        (color.b() as f32 * factor).round().min(255.0) as u8,
        color.a(),
    )
}
