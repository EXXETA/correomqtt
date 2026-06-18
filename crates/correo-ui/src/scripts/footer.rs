use egui::Ui;

const SCRIPTING_HELP_URL: &str = "https://github.com/EXXETA/correomqtt/wiki/scripting";

pub(super) fn help_link(ui: &mut Ui) {
    ui.hyperlink_to("Scripting help", SCRIPTING_HELP_URL);
}
