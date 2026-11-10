use correo_plugins::{HookKind, HostSurface, PluginManifest};

#[test]
fn systopic_manifest_declares_a_connection_action_with_mqtt_and_ui_access() {
    let manifest = PluginManifest::from_toml_str(include_str!(
        "../../../plugins/correo-plugins-systopic/plugin.toml"
    ))
    .unwrap();

    assert_eq!(manifest.id, "org.correomqtt.plugins.system-topic");
    assert!(manifest.capabilities.hooks.is_empty());
    assert!(manifest.entrypoint_for(HookKind::DetailFormatter).is_none());
    assert_eq!(manifest.connection_header_actions.len(), 1);
    assert_eq!(manifest.connection_header_actions[0].id, "system-topics");
    for surface in [HostSurface::Mqtt, HostSurface::Ui] {
        assert!(manifest.capabilities.grants_host_surface(surface));
    }
    for surface in [
        HostSurface::Filesystem,
        HostSurface::Network,
        HostSurface::Secrets,
    ] {
        assert!(!manifest.capabilities.grants_host_surface(surface));
    }
}
