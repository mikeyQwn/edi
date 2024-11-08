mod app;
mod escaping;
mod terminal;
mod window;

fn main() {
    let mut app = app::App::new().initialize();
    app.run();
}
