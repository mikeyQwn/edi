use std::os::fd::{AsRawFd, RawFd};

type Result<T> = std::result::Result<T, std::io::Error>;

pub struct Terminal {}

impl Terminal {
    pub fn new() -> Self {
        Self {}
    }

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

    fn get_stdin_fd() -> RawFd {
        std::io::stdin().as_raw_fd()
    }
}
