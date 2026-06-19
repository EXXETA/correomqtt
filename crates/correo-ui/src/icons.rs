use correo_core::Workspace;
use egui::{FontData, FontDefinitions, FontFamily};
use egui_phosphor::regular;

const GEORGIAN_FALLBACK_FONT: &str = "correo-georgian-fallback";
const BUNDLED_GEORGIAN_FONT: &[u8] = include_bytes!("../../../assets/NotoSansGeorgian-Regular.ttf");

pub(crate) fn install(context: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    add_georgian_fallback(&mut fonts);
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    context.set_fonts(fonts);
}

fn add_georgian_fallback(fonts: &mut FontDefinitions) {
    fonts.font_data.insert(
        GEORGIAN_FALLBACK_FONT.to_owned(),
        FontData::from_static(BUNDLED_GEORGIAN_FONT).into(),
    );
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .push(GEORGIAN_FALLBACK_FONT.to_owned());
    }
}

pub(crate) fn workspace_icon(workspace: Workspace) -> &'static str {
    match workspace {
        Workspace::Connections => regular::LIST_BULLETS,
        Workspace::ImportExport => regular::TROLLEY_SUITCASE,
        Workspace::Scripts => regular::SCROLL,
        Workspace::Plugins => regular::PACKAGE,
        Workspace::Diagnostics => regular::BUG,
        Workspace::Settings => regular::GEAR,
        Workspace::About => regular::INFO,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_icons_match_sidebar_spec() {
        assert_eq!(
            workspace_icon(Workspace::Connections),
            regular::LIST_BULLETS
        );
        assert_eq!(
            workspace_icon(Workspace::ImportExport),
            regular::TROLLEY_SUITCASE
        );
        assert_eq!(workspace_icon(Workspace::Scripts), regular::SCROLL);
        assert_eq!(workspace_icon(Workspace::Plugins), regular::PACKAGE);
        assert_eq!(workspace_icon(Workspace::Diagnostics), regular::BUG);
        assert_eq!(workspace_icon(Workspace::Settings), regular::GEAR);
        assert_eq!(workspace_icon(Workspace::About), regular::INFO);
    }
}
