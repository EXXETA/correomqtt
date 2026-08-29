use super::{APP_ID, APP_NAME, BIN_NAME, VENDOR};

pub(super) fn linux_desktop_entry() -> String {
    format!(
        "[Desktop Entry]\n\
         Name={APP_NAME}\n\
         Comment=Native MQTT desktop client\n\
         Exec={BIN_NAME}\n\
         Icon={APP_ID}\n\
         StartupWMClass={APP_ID}\n\
         Terminal=false\n\
         Type=Application\n\
         Categories=Development;Network;\n"
    )
}

pub(super) fn linux_metainfo() -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <component type=\"desktop-application\">\n\
           <id>{APP_ID}</id>\n\
           <name>{APP_NAME}</name>\n\
           <summary>Native MQTT desktop client</summary>\n\
           <metadata_license>CC0-1.0</metadata_license>\n\
           <project_license>GPL-3.0-or-later</project_license>\n\
         </component>\n"
    )
}

pub(super) fn macos_info_plist() -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
         \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\">\n\
         <dict>\n\
           <key>CFBundleDisplayName</key><string>{APP_NAME}</string>\n\
           <key>CFBundleExecutable</key><string>{BIN_NAME}</string>\n\
           <key>CFBundleIconFile</key><string>Icon.icns</string>\n\
           <key>CFBundleIdentifier</key><string>{APP_ID}</string>\n\
           <key>CFBundleName</key><string>{APP_NAME}</string>\n\
           <key>CFBundlePackageType</key><string>APPL</string>\n\
           <key>CFBundleShortVersionString</key><string>{}</string>\n\
           <key>CFBundleVersion</key><string>{}</string>\n\
           <key>LSApplicationCategoryType</key><string>public.app-category.developer-tools</string>\n\
         </dict>\n\
         </plist>\n",
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_VERSION")
    )
}

pub(super) fn windows_metadata() -> String {
    format!(
        "{{\n  \"name\": \"{APP_NAME}\",\n  \"identifier\": \"{APP_ID}\",\n  \
         \"version\": \"{}\",\n  \"vendor\": \"{VENDOR}\",\n  \"binary\": \
         \"{BIN_NAME}.exe\",\n  \"icon\": \"icons/Icon.ico\",\n  \"signed\": false\n}}\n",
        env!("CARGO_PKG_VERSION")
    )
}

pub(super) fn package_readme() -> String {
    format!(
        "{APP_NAME} unsigned beta package\n\n\
         Version: {}\n\
         Vendor: {VENDOR}\n\
         App ID: {APP_ID}\n\n\
         This package is intentionally unsigned. Signing, notarization, \
         auto-update, paid services, and external release commitments are \
         outside this automation scope.\n\n\
         Runtime data:\n\
         Set CORREOMQTT_CONFIG_DIR to use a specific config/history/log root.\n\
         Without it, the Rust beta uses the OS project data directory for \
         org/CorreoMQTT/CorreoMQTT and also checks legacy Java roots during startup.\n\
         Current config and histories live under that root. Script execution \
         metadata/logs live under scripts/executions/ and scripts/logs/ when \
         scripting persistence writes them. Rust plugin packages and \
         local-repo.json are included next to the executable. \
         App diagnostics currently go to stdout/stderr.\n",
        env!("CARGO_PKG_VERSION")
    )
}
