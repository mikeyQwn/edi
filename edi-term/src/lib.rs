//! Terminal state management

pub mod coord;
pub mod escaping;
pub mod input;
pub mod prettify;
pub mod window;

use coord::Dimensions;
use nix::{errno::Errno, ioctl_read_bad, libc::TIOCGWINSZ, sys::termios};

use std::os::fd::{AsRawFd, RawFd};

/// Returns the current state of the terminal
/// May be used to restore the state after manipulating it with the `restore_state` function
///
/// # Errors
///
/// Returns an error with corresponding `Errno` if underlying c function fails
pub fn get_current_state() -> Result<termios::Termios, Errno> {
    termios::tcgetattr(std::io::stdin())
}

/// Puts the stdin into "raw" mode
///
/// It shoud be restored to the initial state, as the "raw" state
/// may persist after the program exits
///
/// # Errors
///
/// Returns an error with corresponding `Errno` if underlying c function fails
pub fn into_raw() -> Result<(), Errno> {
    let mut termios = termios::tcgetattr(std::io::stdin())?;

    termios
        .local_flags
        .remove(termios::LocalFlags::ICANON | termios::LocalFlags::ECHO);
    // termios.local_flags &= !(termios::LocalFlags::ICANON | termios::LocalFlags::ECHO);

    // termios
    //     .input_flags
    //     .remove(termios::InputFlags::IXON | termios::InputFlags::ICRNL);
    //
    termios.output_flags.remove(termios::OutputFlags::OPOST);
    termios.control_flags.remove(termios::ControlFlags::CS8);

    termios.control_chars[nix::libc::VMIN] = 1;
    termios.control_chars[nix::libc::VTIME] = 0;

    termios::tcsetattr(std::io::stdin(), termios::SetArg::TCSAFLUSH, &termios)
}

/// Restores the terminal state to the given state
///
/// # Errors
///
/// Returns an error with corresponding `Errno` if underlying c function fails
pub fn restore_state(state: &termios::Termios) -> Result<(), Errno> {
    termios::tcsetattr(std::io::stdin(), termios::SetArg::TCSAFLUSH, state)
}

ioctl_read_bad!(get_win_size, TIOCGWINSZ, nix::pty::Winsize);

/// Returns the size of the current terminal (columns and rows)
///
/// # Errors
///
/// Returns an error with corresponding `Errno` if underlying c function fails
pub fn get_size() -> Result<Dimensions<u16>, Errno> {
    let mut winsize = nix::pty::Winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    // SAFETY: winsize struct is valid and mutable
    unsafe {
        let _ = get_win_size(get_stdin_fd(), &raw mut winsize)?;
    }

    Ok(Dimensions::new(winsize.ws_col, winsize.ws_row))
}

/// Executes a function within raw mode, ensuring that state is restored after function returns
///
/// # Errors
///
/// Returns an error with corresponding `Errno` if underlying c function fails
pub fn within_raw_mode<T>(f: impl FnOnce() -> T) -> Result<T, Errno> {
    let initial_state = get_current_state()?;
    into_raw()?;

    let ret = f();

    restore_state(&initial_state)?;
    Ok(ret)
}

fn get_stdin_fd() -> RawFd {
    std::io::stdin().as_raw_fd()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn within_raw() {
        let init_state = get_current_state().unwrap();
        let raw_state = within_raw_mode(|| get_current_state().unwrap()).unwrap();

        let exit_state = get_current_state().unwrap();
        assert_eq!(init_state, exit_state);
        assert_ne!(init_state, raw_state);
    }
}
