use std::{
    env, fs, panic,
    path::{Path, PathBuf},
};

use correo_core::ThemeMode;
use correo_ui::CorreoUi;
use egui_kittest::Harness;
use image::RgbaImage;

#[path = "support/migration_recovery_scenarios.rs"]
mod migration_recovery_scenarios;
#[path = "support/screenshot_fallback.rs"]
mod screenshot_fallback;
#[path = "support/screenshot_scenarios.rs"]
mod screenshot_scenarios;
#[path = "support/software_renderer.rs"]
mod software_renderer;

use migration_recovery_scenarios::{recovery_captures, snapshot_for as recovery_snapshot_for};
use screenshot_scenarios::{mode_slug, screenshot_captures, snapshot_for, Capture};

#[test]
fn captures_visual_qa_matrix() {
    clear_existing_screenshots();

    let mut artifacts: Vec<_> = screenshot_captures().into_iter().map(capture).collect();
    artifacts.extend(recovery_captures().into_iter().map(capture_recovery));

    write_manifest(&artifacts);
}

struct ScreenshotArtifact {
    file_name: String,
    surface: String,
    mode: ThemeMode,
    size: (u32, u32),
    renderer: Renderer,
}

#[derive(Clone, Copy)]
enum Renderer {
    Live,
    Fallback,
}

fn capture(capture: Capture) -> ScreenshotArtifact {
    let (image, renderer) = match render_egui_shell(&capture) {
        Ok(image) => {
            eprintln!("screenshot renderer=live file={}", capture.file_name);
            (image, Renderer::Live)
        }
        Err(error) if fallback_allowed(&error) => {
            eprintln!(
                "screenshot renderer=fallback file={} reason={error}",
                capture.file_name
            );
            (
                screenshot_fallback::fallback_shell_capture(capture.clone()),
                Renderer::Fallback,
            )
        }
        Err(error) => {
            panic!(
                "live screenshot renderer failed for {}: {error}. Set \
                 CORREO_SCREENSHOT_ALLOW_FALLBACK=1 only when recording explicit fallback artifacts \
                 for non-adapter failures.",
                capture.file_name
            );
        }
    };
    assert_eq!(image.dimensions(), capture.size);

    let output_dir = screenshot_dir();
    fs::create_dir_all(&output_dir).expect("screenshot directory should be creatable");
    image
        .save(output_dir.join(&capture.file_name))
        .expect("screenshot should be writable");

    ScreenshotArtifact {
        file_name: capture.file_name,
        surface: capture.scenario.label().to_owned(),
        mode: capture.mode.clone(),
        size: capture.size,
        renderer,
    }
}

fn capture_recovery(capture: migration_recovery_scenarios::RecoveryCapture) -> ScreenshotArtifact {
    let (image, renderer) = render_recovery_shell(&capture);
    assert_eq!(image.dimensions(), capture.size);

    let output_dir = screenshot_dir();
    fs::create_dir_all(&output_dir).expect("screenshot directory should be creatable");
    image
        .save(output_dir.join(&capture.file_name))
        .expect("screenshot should be writable");

    ScreenshotArtifact {
        file_name: capture.file_name,
        surface: capture.scenario.label().to_owned(),
        mode: capture.mode.clone(),
        size: capture.size,
        renderer,
    }
}

fn render_egui_shell(capture: &Capture) -> Result<RgbaImage, String> {
    render_egui_snapshot_at_zoom(snapshot_for(capture), capture.size, capture.zoom_factor)
}

fn render_recovery_shell(
    capture: &migration_recovery_scenarios::RecoveryCapture,
) -> (RgbaImage, Renderer) {
    match render_egui_snapshot(recovery_snapshot_for(capture), capture.size) {
        Ok(image) => {
            eprintln!("screenshot renderer=live file={}", capture.file_name);
            (image, Renderer::Live)
        }
        Err(error) => {
            panic!(
                "live screenshot renderer failed for {}: {error}",
                capture.file_name
            );
        }
    }
}

fn render_egui_snapshot(
    snapshot: correo_core::AppSnapshot,
    size: (u32, u32),
) -> Result<RgbaImage, String> {
    render_egui_snapshot_at_zoom(snapshot, size, 1.0)
}

fn render_egui_snapshot_at_zoom(
    snapshot: correo_core::AppSnapshot,
    size: (u32, u32),
    zoom_factor: f32,
) -> Result<RgbaImage, String> {
    let hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    let rendered = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let mut shell = CorreoUi::for_snapshot(snapshot);
        let mut harness = Harness::builder()
            .with_size(egui::vec2(size.0 as f32, size.1 as f32))
            .with_pixels_per_point(1.0)
            .renderer(software_renderer::SoftwareRenderer::default())
            .build(move |ctx| {
                ctx.set_debug_on_hover(false);
                ctx.set_zoom_factor(zoom_factor);
                ctx.all_styles_mut(|style| {
                    style.debug.debug_on_hover = false;
                    style.debug.debug_on_hover_with_all_modifiers = false;
                });
                shell.draw(ctx);
            });

        harness.run();
        harness.render()
    }));

    panic::set_hook(hook);
    rendered.map_err(|payload| {
        format!(
            "offscreen renderer panicked before producing an image: {}",
            panic_message(payload)
        )
    })?
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_owned()
    }
}

fn fallback_allowed(error: &str) -> bool {
    explicit_fallback_allowed() || missing_graphics_adapter(error)
}

fn explicit_fallback_allowed() -> bool {
    env::var("CORREO_SCREENSHOT_ALLOW_FALLBACK").is_ok_and(|value| value == "1")
}

fn missing_graphics_adapter(error: &str) -> bool {
    error.contains("NoSuitableAdapterFound") || error.contains("No adapter found")
}

fn write_manifest(artifacts: &[ScreenshotArtifact]) {
    let output_dir = screenshot_dir();
    fs::create_dir_all(&output_dir).expect("screenshot directory should be creatable");
    let mut manifest = String::from("# CorreoMQTT screenshot manifest\n\n");
    manifest.push_str("## Coverage\n\n");
    manifest.push_str(
        "- Empty connections and active workbench cover light/dark at 1280x800, 1024x768, and 900x640; active workbench also covers compact 720x640.\n",
    );
    manifest.push_str(
        "- Active workbench message rows include a 150% zoom capture to check font-metric row height and baseline alignment.\n",
    );
    manifest.push_str(
        "- Connection transfer covers .cqc import choose/password/review/outcome states and export \
         plain/encrypted/path/outcome states; CQM message actions are covered in the workbench.\n",
    );
    manifest
        .push_str("- Global settings covers Appearance, Language, Search, and Keyring sections.\n");
    manifest.push_str(
        "- Plugin Manager covers light/dark at 1280x800, 1024x700, and 900x600, plus fixture \
         states for loading, empty, disable confirmation, WASM load error, hook config validation, \
         and filtered diagnostics.\n",
    );
    manifest.push_str(
        "- Migration recovery covers detection, password needed/error, review warnings, partial \
         success, failure after write, restore confirmation, and restore failure.\n\n",
    );
    manifest.push_str("## Artifacts\n\n");
    for artifact in artifacts {
        let renderer = match artifact.renderer {
            Renderer::Live => "live",
            Renderer::Fallback => "fallback",
        };
        manifest.push_str(&format!(
            "- file: {}\n  surface: {}\n  mode: {}\n  size: {}x{}\n  renderer: {renderer}\n",
            artifact.file_name,
            artifact.surface,
            mode_slug(&artifact.mode),
            artifact.size.0,
            artifact.size.1
        ));
    }

    fs::write(output_dir.join("manifest.md"), manifest).expect("screenshot manifest is writable");
}

fn screenshot_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/correomqtt-screenshots")
        .canonicalize()
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/correomqtt-screenshots")
        })
}

fn clear_existing_screenshots() {
    let output_dir = screenshot_dir();
    if !output_dir.exists() {
        return;
    }

    for entry in fs::read_dir(&output_dir).expect("screenshot directory should be readable") {
        let path = entry.expect("screenshot entry should be readable").path();
        if is_screenshot_artifact(&path) {
            fs::remove_file(path).expect("old screenshot should be removable");
        }
    }
}

fn is_screenshot_artifact(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("png")
        || path.file_name().and_then(|name| name.to_str()) == Some("manifest.md")
}
