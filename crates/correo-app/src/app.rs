use correo_core::{
    AppRuntime, Diagnostic, HistoryPersistenceWorker, MigrationPersistenceWorker, MqttService,
    PluginHookExecutor, RumqttSessionFactory, ScriptingWorker, SettingsPersistenceWorker,
};
use std::sync::Arc;

use crate::plugins::{InstalledPluginExecutor, PluginFileInstaller};
use crate::startup::{history_root, load_startup_state};

pub fn run() -> eframe::Result {
    prefer_x11_when_wayland_is_unstable();
    correo_diagnostics::install_tracing();
    tracing::info!("starting CorreoMQTT desktop shell");

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_app_id("org.correomqtt.CorreoMQTT")
            .with_title("CorreoMQTT")
            .with_icon(app_icon())
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([550.0, 520.0]),
        ..Default::default()
    };

    eframe::run_native(
        "CorreoMQTT",
        options,
        Box::new(|creation_context| Ok(Box::new(CorreoDesktopApp::new(creation_context)))),
    )
}

fn prefer_x11_when_wayland_is_unstable() {
    #[cfg(target_os = "linux")]
    {
        let user_selected_backend = std::env::var_os("WINIT_UNIX_BACKEND").is_some();
        let allow_wayland = std::env::var_os("CORREOMQTT_ALLOW_WAYLAND").is_some();
        let wayland_available = std::env::var_os("WAYLAND_DISPLAY").is_some();
        let x11_available = std::env::var_os("DISPLAY").is_some();

        if !user_selected_backend && !allow_wayland && wayland_available && x11_available {
            std::env::set_var("WINIT_UNIX_BACKEND", "x11");
        }
    }
}

fn app_icon() -> eframe::egui::IconData {
    eframe::icon_data::from_png_bytes(include_bytes!("../../../assets/package/Icon.png"))
        .unwrap_or_default()
}

struct CorreoDesktopApp {
    runtime: AppRuntime,
    mqtt_runtime: Option<tokio::runtime::Runtime>,
    ui: correo_ui::CorreoUi,
}

impl CorreoDesktopApp {
    fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        let theme_mode = correo_ui::stored_theme(creation_context);
        let loaded = load_startup_state(theme_mode);
        let mut runtime = AppRuntime::with_startup_state(loaded.state);
        let storage_root = history_root();
        runtime.attach_plugin_installer(PluginFileInstaller::new(storage_root.clone()));
        let plugin_executor = attach_plugin_executor(
            &mut runtime,
            storage_root.clone(),
            &loaded.plugins.installed_package_dirs,
        );
        let mqtt_runtime = attach_mqtt_service(&mut runtime);
        runtime.attach_history_worker(HistoryPersistenceWorker::start(storage_root.clone()));
        runtime.attach_migration_worker(MigrationPersistenceWorker::start(storage_root.clone()));
        runtime.attach_settings_worker(SettingsPersistenceWorker::start(storage_root.clone()));
        runtime.attach_scripting_worker(ScriptingWorker::start_with_mqtt_sender(
            storage_root,
            runtime.mqtt_command_sender(),
        ));
        let ui = correo_ui::CorreoUi::with_command_sender(
            creation_context,
            runtime.snapshot().clone(),
            runtime.command_sender(),
            plugin_executor.map(|executor| {
                Arc::new(move |text: &str, active_plugin_ids: &[String]| {
                    executor.highlight_payload(text, active_plugin_ids)
                }) as correo_ui::PayloadHighlighter
            }),
        );
        Self {
            runtime,
            mqtt_runtime,
            ui,
        }
    }

    fn pump_runtime(&mut self, context: &eframe::egui::Context) {
        let report = self.runtime.pump();
        if report.snapshot_changed {
            self.ui.set_snapshot(self.runtime.snapshot().clone());
            context.request_repaint();
        }
        if report.shutdown_requested {
            context.send_viewport_cmd(eframe::egui::ViewportCommand::Close);
        }
    }
}

impl Drop for CorreoDesktopApp {
    fn drop(&mut self) {
        if let Some(mqtt_runtime) = &self.mqtt_runtime {
            mqtt_runtime.block_on(self.runtime.shutdown_mqtt());
        }
    }
}

fn attach_plugin_executor(
    runtime: &mut AppRuntime,
    config_root: std::path::PathBuf,
    package_dirs: &[std::path::PathBuf],
) -> Option<Arc<dyn PluginHookExecutor>> {
    match InstalledPluginExecutor::load(config_root, package_dirs) {
        Ok(executor) => {
            let executor: Arc<dyn PluginHookExecutor> = Arc::new(executor);
            runtime.attach_plugin_hook_executor(executor.clone());
            Some(executor)
        }
        Err(error) => {
            record_startup_diagnostic(
                runtime,
                format!("Plugin runtime could not load installed plugins: {error}"),
            );
            None
        }
    }
}

fn attach_mqtt_service(runtime: &mut AppRuntime) -> Option<tokio::runtime::Runtime> {
    let mqtt_runtime = match tokio::runtime::Builder::new_multi_thread()
        .thread_name("correo-mqtt")
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            record_startup_diagnostic(
                runtime,
                format!("MQTT runtime could not be started: {error}"),
            );
            return None;
        }
    };

    let service = {
        let _guard = mqtt_runtime.enter();
        MqttService::spawn(RumqttSessionFactory)
    };
    match service {
        Ok(service) => runtime.attach_mqtt_service(service),
        Err(error) => {
            record_startup_diagnostic(
                runtime,
                format!("MQTT service could not be started: {error}"),
            );
        }
    }
    Some(mqtt_runtime)
}

fn record_startup_diagnostic(runtime: &mut AppRuntime, message: String) {
    let _ = runtime
        .event_sender()
        .emit(correo_core::AppEvent::DiagnosticRaised(Diagnostic::error(
            message,
        )));
    runtime.pump();
}

impl eframe::App for CorreoDesktopApp {
    fn update(&mut self, context: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.pump_runtime(context);
        self.ui.draw(context);
        self.pump_runtime(context);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(
            storage,
            correo_ui::THEME_KEY,
            &self.runtime.snapshot().theme_mode,
        );
    }
}
