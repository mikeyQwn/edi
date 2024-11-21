use std::{
    io::Read,
    os::fd::AsFd,
    sync::mpsc::{Receiver, RecvError, SendError, Sender},
};

use thiserror::Error;

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
    Backspace,

    Unimplemented(Vec<u8>),
}

/// A stream of input events
/// This struct is used to read input from a file descriptor
/// and convert it into a stream of input events
/// The stream can be read from using the `recv` method
pub struct InputStream {
    kill: Sender<()>,
    events: Receiver<Message>,
}

impl InputStream {
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
        let mut buffer = [0_u8; 4];
        let mut reader = timeout_readwrite::TimeoutReader::new(input_handle, None);

        let (t_events, r_events) = std::sync::mpsc::channel();
        let (t_kill, r_kill) = std::sync::mpsc::channel();

        std::thread::spawn(move || loop {
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

            let event = match buffer[0] {
                _ if n > 1 => Input::Unimplemented(buffer[..n].into()),
                127 => Input::Backspace,
                c if c.is_ascii() => Input::Keypress(c as char),
                _ => Input::Unimplemented(buffer[..n].into()),
            };

            if let Err(e) = t_events.send(Message::Input(event)) {
                let _ = t_events.send(Message::Error(InputError::from(Box::new(e))));
            }
        });

        (r_events, t_kill)
    }
}

impl Drop for InputStream {
    fn drop(&mut self) {
        let _ = self.kill.send(());
    }
}
