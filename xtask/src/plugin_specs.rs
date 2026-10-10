#[derive(Debug, Clone, Copy)]
pub struct PluginBuildSpec {
    pub package: &'static str,
    #[allow(dead_code)]
    pub crate_path: &'static str,
    pub manifest_path: &'static str,
    pub wasm_stem: &'static str,
}

pub const PLUGIN_SPECS: &[PluginBuildSpec] = &[
    PluginBuildSpec {
        package: "correo-plugin-base64",
        crate_path: "crates/correo-plugin-base64",
        manifest_path: "crates/correo-plugin-base64/plugin.toml",
        wasm_stem: "correo_plugin_base64",
    },
    PluginBuildSpec {
        package: "correo-plugin-zip-manipulator",
        crate_path: "crates/correo-plugin-zip-manipulator",
        manifest_path: "crates/correo-plugin-zip-manipulator/plugin.toml",
        wasm_stem: "correo_plugin_zip_manipulator",
    },
    PluginBuildSpec {
        package: "correo-plugins-advanced-validator",
        crate_path: "plugins/correo-plugins-advanced-validator",
        manifest_path: "plugins/correo-plugins-advanced-validator/plugin.toml",
        wasm_stem: "correo_plugins_advanced_validator",
    },
    PluginBuildSpec {
        package: "correo-plugins-contains-string-validator",
        crate_path: "plugins/correo-plugins-contains-string-validator",
        manifest_path: "plugins/correo-plugins-contains-string-validator/plugin.toml",
        wasm_stem: "correo_plugins_contains_string_validator",
    },
    PluginBuildSpec {
        package: "correo-plugins-json-format",
        crate_path: "plugins/correo-plugins-json-format",
        manifest_path: "plugins/correo-plugins-json-format/plugin.toml",
        wasm_stem: "correo_plugins_json_format",
    },
    PluginBuildSpec {
        package: "correo-plugins-systopic",
        crate_path: "plugins/correo-plugins-systopic",
        manifest_path: "plugins/correo-plugins-systopic/plugin.toml",
        wasm_stem: "correo_plugins_systopic",
    },
    PluginBuildSpec {
        package: "correo-plugins-xml-xsd-validator",
        crate_path: "plugins/correo-plugins-xml-xsd-validator",
        manifest_path: "plugins/correo-plugins-xml-xsd-validator/plugin.toml",
        wasm_stem: "correo_plugins_xml_xsd_validator",
    },
    PluginBuildSpec {
        package: "correo-plugin-xml-format",
        crate_path: "plugins/xml-format",
        manifest_path: "plugins/xml-format/plugin.toml",
        wasm_stem: "correo_plugin_xml_format",
    },
    PluginBuildSpec {
        package: "correo-plugin-save-manipulator",
        crate_path: "plugins/save-manipulator",
        manifest_path: "plugins/save-manipulator/plugin.toml",
        wasm_stem: "correo_plugin_save_manipulator",
    },
];
