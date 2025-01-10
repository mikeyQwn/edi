//! Terminal state management

pub mod escaping;
pub mod input;
pub mod window;

use std::{
    io::Result,
    os::fd::{AsRawFd, RawFd},
};

use crate::vec2::Vec2;

/// Returns the current state of the terminal
/// May be used to restore the state after manipulating it with the `restore_state` function
///
/// # Errors
/// Returns an `io::Error` if underlying c function fails
pub fn get_current_state() -> Result<termios::Termios> {
    termios::Termios::from_fd(get_stdin_fd())
}

/// Puts the stdin into "raw" mode
///
/// It shoud be restored to the initial state, as the "raw" state
/// may persist after the program exits
///
/// # Errors
/// Returns an `io::Error` if underlying c functions fails
pub fn into_raw() -> Result<()> {
    let fd = get_stdin_fd();
    let mut termios = termios::Termios::from_fd(fd)?;
    termios.c_lflag &= !(termios::ICANON | termios::ECHO);
    termios::tcsetattr(fd, termios::TCSAFLUSH, &termios)
}

/// Restores the terminal state to the given state
///
/// # Errors
/// Returns an `io::Error` if underlying c function fails
pub fn restore_state(state: &termios::Termios) -> Result<()> {
    termios::tcsetattr(get_stdin_fd(), termios::TCSAFLUSH, state)
}

/// Returns the size of the current terminal (columns and rows)
///
/// # Errors
/// Returns an `io::Error` if underlying c function fails
pub fn get_size() -> Result<Vec2<u16>> {
    let mut winsize = libc::winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    // Safety: `winsize` is a valid `libc::winsize` struct.
    // `TIOCGWINSZ` is a valid ioctl request for a terminal file descriptor.
    // `winsize` is a valid pointer to a mutable `libc::winsize` struct.
    // The return value of `ioctl` is checked for errors.
    unsafe {
        let ok = libc::ioctl(get_stdin_fd(), libc::TIOCGWINSZ, &mut winsize);
        if ok == -1 {
            return Err(std::io::Error::last_os_error());
        }
    }

    Ok(Vec2::new(winsize.ws_col, winsize.ws_row))
}

fn get_stdin_fd() -> RawFd {
    std::io::stdin().as_raw_fd()
}
