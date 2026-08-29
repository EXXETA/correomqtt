mod app;
mod builtin_broker;
mod plugins;
mod startup;
mod update_check;

fn main() -> eframe::Result {
    if std::env::args().nth(1).as_deref() == Some(correo_core::builtin_broker_child_arg()) {
        std::process::exit(builtin_broker::run_child());
    }
    app::run()
}
