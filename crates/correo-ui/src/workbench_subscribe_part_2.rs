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
