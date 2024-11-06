use terminal::Terminal;

mod terminal;
mod window;

const WIN_SIZE: (usize, usize) = (10, 10);

fn main() {
    let current_state = terminal::Terminal::get_current_state().unwrap();
    Terminal::into_raw().unwrap();
    Terminal::clear_screen().unwrap();

    let mut window = window::Window::new();
    window.resize(WIN_SIZE.0, WIN_SIZE.1);

    for y in 0..WIN_SIZE.1 {
        for x in 0..WIN_SIZE.0 {
            window.put_char(x, y, 'X');
            window.render().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    terminal::Terminal::restore_state(&current_state).unwrap();
}
