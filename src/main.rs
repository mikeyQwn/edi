use terminal::Terminal;

mod app;
mod terminal;
mod window;

fn main() {
    let mut app = app::App::new().initialize();
    app.run();
}
