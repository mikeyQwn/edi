use std::{
    io::Write,
    os::fd::{AsRawFd, RawFd},
};

use crate::log;

type Result<T> = std::result::Result<T, std::io::Error>;

pub struct Terminal;

impl Terminal {
    pub fn get_current_state() -> Result<termios::Termios> {
        let fd = Self::get_stdin_fd();
        termios::Termios::from_fd(fd)
    }

    pub fn into_raw() -> Result<()> {
        let fd = Self::get_stdin_fd();
        let mut termios = termios::Termios::from_fd(fd)?;
        termios.c_lflag &= !(termios::ICANON | termios::ECHO);
        termios::tcsetattr(fd, termios::TCSAFLUSH, &termios)?;
        Ok(())
    }

    pub fn restore_state(state: &termios::Termios) -> Result<()> {
        let fd = Self::get_stdin_fd();
        termios::tcsetattr(fd, termios::TCSAFLUSH, state)?;
        Ok(())
    }

    pub fn get_size() -> Result<(u16, u16)> {
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
            let ok = libc::ioctl(Self::get_stdin_fd(), libc::TIOCGWINSZ, &mut winsize);
            if ok == -1 {
                return Err(std::io::Error::last_os_error());
            }
        }

        //log::debug!("terminal size is {} {}", winsize.ws_col, winsize.ws_row);

        Ok((winsize.ws_col, winsize.ws_row))
    }

    pub fn flush() -> Result<()> {
        std::io::stdout().flush()
    }

    fn get_stdin_fd() -> RawFd {
        std::io::stdin().as_raw_fd()
    }
}
