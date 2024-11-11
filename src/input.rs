use std::{io::Read, os::fd::AsFd, sync::mpsc::Receiver, sync::mpsc::Sender};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum InputError {
    #[error("error while reading: `{0}`")]
    IO(#[from] std::io::Error),
    #[error("unable to send to a channel: `{0}`")]
    Send(#[from] std::sync::mpsc::SendError<InputEvent>),
}

#[derive(Clone, Copy, Debug)]
pub enum InputEvent {
    Keypress(char),
    Backspace,

    Unimplemented([u8; 4], u8),
}

pub fn to_event_stream<H>(
    input_handle: H,
) -> (Receiver<InputEvent>, Receiver<InputError>, Sender<()>)
where
    H: Read + AsFd + Send + 'static,
{
    let mut buffer = [0_u8; 4];
    let mut reader = timeout_readwrite::TimeoutReader::new(
        input_handle,
        Some(std::time::Duration::from_millis(200)),
    );

    let (t_events, r_events) = std::sync::mpsc::channel();
    let (t_errors, r_errors) = std::sync::mpsc::channel();
    let (t_kill, r_kill) = std::sync::mpsc::channel();

    std::thread::spawn(move || loop {
        let n = match reader.read(&mut buffer) {
            Ok(n) => n,
            Err(e) => {
                let _ = t_errors.send(InputError::from(e));
                continue;
            }
        };
        if r_kill.try_recv().is_ok() {
            break;
        }

        let event = match buffer[0] {
            _ if n > 1 => InputEvent::Unimplemented(buffer, n as u8),
            127 => InputEvent::Backspace,
            c if c.is_ascii() => InputEvent::Keypress(c as char),
            _ => InputEvent::Unimplemented(buffer, n as u8),
        };

        if let Err(e) = t_events.send(event) {
            let _ = t_errors.send(InputError::from(e));
        }
    });

    (r_events, r_errors, t_kill)
}
