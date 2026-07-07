mod app;
mod plugins;
mod startup;
mod update_check;

fn main() -> eframe::Result {
    app::run()
}
