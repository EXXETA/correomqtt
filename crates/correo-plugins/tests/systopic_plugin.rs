use correo_plugins::{HostSurface, PluginManifest};

// The $SYS metrics feature is a built-in host window: the plugin declares no
// WASM hooks but needs the MQTT and UI host surfaces plus a connection header
// action. The metric labeling logic is unit-tested inside the systopic crate.
#[test]
fn systopic_manifest_declares_header_action_with_mqtt_and_ui_caps() {
    let manifest = PluginManifest::from_toml_str(include_str!(
        "../../../plugins/correo-plugins-systopic/plugin.toml"
    ))
    .unwrap();

    assert_eq!(manifest.id, "org.correomqtt.plugins.system-topic");
    assert!(manifest.capabilities.hooks.is_empty());
    assert!(manifest.capabilities.grants_host_surface(HostSurface::Mqtt));
    assert!(manifest.capabilities.grants_host_surface(HostSurface::Ui));
    for surface in [
        HostSurface::Filesystem,
        HostSurface::MessageSave,
        HostSurface::Network,
        HostSurface::Secrets,
    ] {
        assert!(!manifest.capabilities.grants_host_surface(surface));
    }

    assert_eq!(manifest.connection_header_actions.len(), 1);
    let action = &manifest.connection_header_actions[0];
    assert_eq!(action.id, "system-topics");
    assert!(action.requires_connected);
}
