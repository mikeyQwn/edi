use std::{
    io::Read,
    os::fd::AsFd,
    sync::mpsc::{Receiver, RecvError, SendError, Sender},
};

use thiserror::Error;

use crate::log;

#[derive(Error, Debug)]
pub enum InputError {
    #[error("error while reading: `{0}`")]
    IO(#[from] std::io::Error),
    #[error("unable to send to a channel: `{0}`")]
    Send(#[from] Box<SendError<Message>>),
    #[error("unable to receive from a channel: `{0}`")]
    Receive(#[from] RecvError),
}

#[derive(Debug)]
pub enum Message {
    Input(Input),
    Error(InputError),
}

#[derive(Clone, Debug)]
pub enum Input {
    Keypress(char),
    Escape,
    Enter,
    Backspace,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,

    #[allow(unused)]
    Unimplemented(Vec<u8>),
}

/// A stream of input events
/// This struct is used to read input from a file descriptor
/// and convert it into a stream of input events
/// The stream can be read from using the `recv` method
pub struct Stream {
    kill: Sender<()>,
    events: Receiver<Message>,
}

impl Stream {
    pub fn from_read<H>(input_handle: H) -> Self
    where
        H: Read + AsFd + Send + 'static,
    {
        let (events, kill) = Self::to_event_stream(input_handle);
        Self { kill, events }
    }

    pub fn recv(&self) -> Result<Message, RecvError> {
        self.events.recv()
    }

    fn to_event_stream<H>(input_handle: H) -> (Receiver<Message>, Sender<()>)
    where
        H: Read + AsFd + Send + 'static,
    {
        let mut reader = timeout_readwrite::TimeoutReader::new(input_handle, None);

        let (t_events, r_events) = std::sync::mpsc::channel();
        let (t_kill, r_kill) = std::sync::mpsc::channel();

        std::thread::spawn(move || loop {
            let mut buffer = [0_u8; 4];
            let n = match reader.read(&mut buffer) {
                Ok(n) => n,
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::TimedOut {
                        continue;
                    }
                    let _ = t_events.send(Message::Error(InputError::from(e)));
                    continue;
                }
            };

            if r_kill.try_recv().is_ok() {
                break;
            }

            let event = match buffer {
                [127, _, _, _] => Input::Backspace,
                [27, 91, 65, _] => Input::ArrowUp,
                [27, 91, 66, _] => Input::ArrowDown,
                [27, 91, 67, _] => Input::ArrowRight,
                [27, 91, 68, _] => Input::ArrowLeft,
                [27, _, _, _] => Input::Escape,
                [10, _, _, _] => Input::Enter,
                [c, _, _, _] if c.is_ascii() => Input::Keypress(c as char),
                _ => Input::Unimplemented(buffer[..n].into()),
            };

            if let Err(e) = t_events.send(Message::Input(event)) {
                let _ = t_events.send(Message::Error(InputError::from(Box::new(e))));
            }
        });

        (r_events, t_kill)
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        let _ = self.kill.send(());
    }
}
