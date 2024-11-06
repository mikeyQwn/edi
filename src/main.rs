use std::io::Read;

use terminal::Terminal;

mod terminal;

fn main() {
    let state = Terminal::get_current_state().unwrap();
    Terminal::into_raw().unwrap();

    println!("[info] Press any key to continue...");
    std::io::stdin().read(&mut [0]).unwrap();

    Terminal::restore_state(&state).unwrap();
    println!("[info] Restored terminal state");
}
