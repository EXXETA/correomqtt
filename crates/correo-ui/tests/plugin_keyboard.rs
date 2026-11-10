use correo_core::{
    sample_snapshot, AppCommand, AppRuntime, PluginSurfaceTab, ThemeMode, Workspace,
};
use correo_ui::CorreoUi;
use egui::{Id, Key, Modifiers};
use egui_kittest::Harness;

#[test]
fn plugin_manager_keyboard_flow_is_command_driven() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1280.0, 720.0))
        .build_state(draw_shell, TestState::new());

    harness.press_key_modifiers(Modifiers::COMMAND, Key::F);
    harness.run();
    assert!(harness
        .ctx
        .memory(|memory| memory.has_focus(Id::new("plugin-manager-search"))));
    harness
        .ctx
        .memory_mut(|memory| memory.surrender_focus(Id::new("plugin-manager-search")));
    harness.run();

    harness.press_key(Key::ArrowDown);
    harness.run();
    harness.state_mut().pump();
    assert_eq!(
        harness
            .state()
            .runtime
            .snapshot()
            .plugins
            .selected_plugin_id,
        "org.correomqtt.plugins.base64"
    );

    harness.press_key(Key::Space);
    harness.run();
    harness.state_mut().pump();
    assert!(harness
        .state()
        .runtime
        .snapshot()
        .plugins
        .disable_confirmation
        .is_some());

    harness.press_key(Key::Escape);
    harness.run();
    harness.state_mut().pump();
    assert!(harness
        .state()
        .runtime
        .snapshot()
        .plugins
        .disable_confirmation
        .is_none());

    harness
        .state()
        .runtime
        .command_sender()
        .send(AppCommand::SelectPluginSurfaceTab(
            PluginSurfaceTab::Marketplace,
        ))
        .unwrap();
    harness.state_mut().pump();
    harness.run();

    harness
        .state()
        .runtime
        .command_sender()
        .send(AppCommand::SelectMarketplacePlugin(
            "marketplace.schema-validator".to_owned(),
        ))
        .unwrap();
    harness.state_mut().pump();
    harness.run();
    surrender_focus(&harness);
    assert_eq!(
        harness.state().runtime.snapshot().plugins.active_tab,
        PluginSurfaceTab::Marketplace
    );
    assert_eq!(
        harness
            .state()
            .runtime
            .snapshot()
            .plugins
            .selected_marketplace_plugin_id,
        "marketplace.schema-validator"
    );

    harness
        .state()
        .runtime
        .command_sender()
        .send(AppCommand::InstallMarketplacePlugin {
            marketplace_plugin_id: "marketplace.schema-validator".to_owned(),
        })
        .unwrap();
    harness.state_mut().pump();
    assert!(harness
        .state()
        .runtime
        .snapshot()
        .plugins
        .plugins
        .iter()
        .any(|plugin| plugin.id == "marketplace.schema-validator"));
}

fn surrender_focus(harness: &Harness<'_, TestState>) {
    harness.ctx.memory_mut(|memory| {
        if let Some(focused) = memory.focused() {
            memory.surrender_focus(focused);
        }
    });
}

struct TestState {
    runtime: AppRuntime,
    shell: CorreoUi,
}

impl TestState {
    fn new() -> Self {
        let mut snapshot = sample_snapshot(ThemeMode::Light);
        snapshot.active_workspace = Workspace::Plugins;

        let runtime = AppRuntime::with_snapshot(snapshot);
        let shell = CorreoUi::for_snapshot_with_command_sender(
            runtime.snapshot().clone(),
            runtime.command_sender(),
        );
        Self { runtime, shell }
    }

    fn pump(&mut self) {
        self.runtime.pump();
        self.shell.set_snapshot(self.runtime.snapshot().clone());
    }
}

fn draw_shell(context: &egui::Context, state: &mut TestState) {
    state.shell.set_snapshot(state.runtime.snapshot().clone());
    state.shell.draw(context);
}
