use correo_core::{AppCommandSender, AppSnapshot};
use egui::{CentralPanel, Context, Frame, Id, ViewportBuilder, ViewportClass, ViewportId, Window};

use crate::{theme::ThemeTokens, workbench_detail, PayloadHighlighter};

pub(crate) fn open_incoming_message(ctx: &Context, message_id: u32) {
    let mut ids = incoming_ids(ctx);
    if !ids.contains(&message_id) {
        ids.push(message_id);
    }
    ctx.data_mut(|data| {
        data.insert_temp(incoming_ids_id(), ids);
        data.insert_temp(focused_incoming_id(), Some(message_id));
    });
}

pub(crate) fn open_outgoing_message(ctx: &Context, message_id: u32) {
    let mut ids = outgoing_ids(ctx);
    if !ids.contains(&message_id) {
        ids.push(message_id);
    }
    ctx.data_mut(|data| {
        data.insert_temp(outgoing_ids_id(), ids);
        data.insert_temp(focused_outgoing_id(), Some(message_id));
    });
}

pub(crate) fn show(
    ctx: &Context,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    payload_highlighter: Option<&PayloadHighlighter>,
) {
    show_outgoing(ctx, snapshot, tokens, commands, payload_highlighter);
    show_incoming(ctx, snapshot, tokens, commands, payload_highlighter);
}

fn show_incoming(
    ctx: &Context,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    payload_highlighter: Option<&PayloadHighlighter>,
) {
    let focused = ctx
        .data_mut(|data| data.get_temp::<Option<u32>>(focused_incoming_id()))
        .flatten();
    let mut retained = Vec::new();
    for message_id in incoming_ids(ctx) {
        let Some(message) = snapshot
            .workbench
            .messages
            .iter()
            .find(|message| message.id == message_id)
        else {
            continue;
        };
        let title = format!("Message {}", message.topic);
        let close_requested = show_message_viewport(
            ctx,
            viewport_id("workbench-message-viewport", message_id),
            Id::new(("workbench-message-window", message_id)),
            &title,
            [520.0, 360.0],
            focused == Some(message_id),
            tokens,
            |ui| {
                workbench_detail::message_window_content(
                    ui,
                    snapshot,
                    message,
                    tokens,
                    commands,
                    payload_highlighter,
                );
            },
        );
        if !close_requested {
            retained.push(message_id);
        }
    }
    ctx.data_mut(|data| {
        data.insert_temp(incoming_ids_id(), retained);
        data.insert_temp(focused_incoming_id(), None::<u32>);
    });
}

fn show_outgoing(
    ctx: &Context,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    payload_highlighter: Option<&PayloadHighlighter>,
) {
    let focused = ctx
        .data_mut(|data| data.get_temp::<Option<u32>>(focused_outgoing_id()))
        .flatten();
    let mut retained = Vec::new();
    for message_id in outgoing_ids(ctx) {
        let Some(row) = snapshot
            .workbench
            .publish
            .history
            .iter()
            .find(|row| row.id == message_id)
        else {
            continue;
        };
        let title = format!("Published {}", row.topic);
        let close_requested = show_message_viewport(
            ctx,
            viewport_id("workbench-outgoing-message-viewport", message_id),
            Id::new(("workbench-outgoing-message-window", message_id)),
            &title,
            [480.0, 260.0],
            focused == Some(message_id),
            tokens,
            |ui| {
                workbench_detail::outgoing_window_content(
                    ui,
                    snapshot,
                    row,
                    tokens,
                    commands,
                    payload_highlighter,
                );
            },
        );
        if !close_requested {
            retained.push(message_id);
        }
    }
    ctx.data_mut(|data| {
        data.insert_temp(outgoing_ids_id(), retained);
        data.insert_temp(focused_outgoing_id(), None::<u32>);
    });
}

fn show_message_viewport(
    ctx: &Context,
    viewport_id: ViewportId,
    embedded_window_id: Id,
    title: &str,
    size: [f32; 2],
    focused: bool,
    tokens: ThemeTokens,
    mut add_contents: impl FnMut(&mut egui::Ui),
) -> bool {
    let mut builder = ViewportBuilder::default()
        .with_title(title)
        .with_inner_size(size)
        .with_min_inner_size([360.0, 220.0]);
    if focused {
        builder = builder.with_active(true);
    }

    ctx.show_viewport_immediate(viewport_id, builder, |ctx, class| {
        let close_requested = ctx.input(|input| input.viewport().close_requested());
        if class == ViewportClass::Embedded {
            let mut open = !close_requested;
            Window::new(title)
                .id(embedded_window_id)
                .open(&mut open)
                .default_size(size)
                .show(ctx, |ui| add_contents(ui));
            close_requested || !open
        } else {
            CentralPanel::default()
                .frame(
                    Frame::NONE
                        .fill(tokens.window_bg)
                        .inner_margin(egui::Margin::same(correo_style::layout::CENTRAL_MARGIN)),
                )
                .show(ctx, |ui| add_contents(ui));
            close_requested
        }
    })
}

fn viewport_id(kind: &'static str, message_id: u32) -> ViewportId {
    ViewportId::from_hash_of((kind, message_id))
}

fn incoming_ids(ctx: &Context) -> Vec<u32> {
    ctx.data_mut(|data| data.get_temp(incoming_ids_id()).unwrap_or_default())
}

fn outgoing_ids(ctx: &Context) -> Vec<u32> {
    ctx.data_mut(|data| data.get_temp(outgoing_ids_id()).unwrap_or_default())
}

fn incoming_ids_id() -> Id {
    Id::new("workbench-open-incoming-message-windows")
}

fn outgoing_ids_id() -> Id {
    Id::new("workbench-open-outgoing-message-windows")
}

fn focused_incoming_id() -> Id {
    Id::new("workbench-focused-incoming-message-window")
}

fn focused_outgoing_id() -> Id {
    Id::new("workbench-focused-outgoing-message-window")
}
