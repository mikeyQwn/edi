//! Raw mode terminal input hadnler implementation

use std::{
    io::Read,
    os::fd::AsFd,
    sync::mpsc::{Receiver, RecvError, SendError, Sender},
};

use thiserror::Error;

use crate::debug;

/// An error that occurs during reading from stdio/sending the input signals through channels
#[derive(Error, Debug)]
pub enum InputError {
    /// Occurs when io reads fail
    #[error("error while reading: `{0}`")]
    IO(#[from] std::io::Error),
    /// Occurs when send to event channel fails
    #[error("unable to send to a channel: `{0}`")]
    Send(#[from] Box<SendError<Message>>),
    /// Occurs when receive from event channel fails
    #[error("unable to receive from a channel: `{0}`")]
    Receive(#[from] RecvError),
}

/// A message sent through the event channel
#[derive(Debug)]
pub enum Message {
    /// A received input
    Input(Input),
    /// An error while reading from the file
    /// The caller might use this error to signal the read stream to stop
    Error(InputError),
}

/// An input receieved in the raw terminal mode
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Input {
    /// A keypress that can be represented with a single ascii character
    Keypress(char),
    /// Simmilar to keypress, but with the ctrl key held
    Control(char),
    /// Esc key
    Escape,
    /// Enter key
    Enter,
    /// Backspace key
    Backspace,
    /// Arrow up
    ArrowUp,
    /// Arrow down
    ArrowDown,
    /// Arrow left
    ArrowLeft,
    /// Arrow right
    ArrowRight,

    /// Inputs for which the handlers are yet to be imlemented
    #[allow(unused)]
    Unimplemented(Vec<u8>),
}

impl Input {
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        match bytes {
            [4] => Input::Control('d'),
            [10] => Input::Enter,
            [18] => Input::Control('r'),
            [21] => Input::Control('u'),
            [27] => Input::Escape,
            [127] => Input::Backspace,
            [c] if c.is_ascii() => Input::Keypress(*c as char),

            [27, 91, 65] => Input::ArrowUp,
            [27, 91, 66] => Input::ArrowDown,
            [27, 91, 67] => Input::ArrowRight,
            [27, 91, 68] => Input::ArrowLeft,

            _ => Input::Unimplemented(bytes.into()),
        }
    }
}

/// A stream of input events
///
/// This struct is used to read input from a file descriptor
/// and convert it into a stream of input events
///
/// The stream can be read from using the `recv` method
#[derive(Debug)]
pub struct Stream {
    kill: Sender<()>,
    events: Receiver<Message>,
}

impl Stream {
    /// Initiates an input stream from stdin
    #[must_use]
    pub fn from_stdin() -> Self {
        Self::from_read(std::io::stdin())
    }

    /// Transforms anything that implements `Read` and `AsFd` into an event stream
    ///
    /// You may not want to use this with anything but the `stdin()`, though
    #[must_use]
    pub fn from_read<H>(input_handle: H) -> Self
    where
        H: Read + AsFd + Send + 'static,
    {
        let (events, kill) = Self::to_event_stream(input_handle);
        Self { kill, events }
    }

    /// Receive a single input event. A call to recv blocks indefinitely
    ///
    /// # Errors
    ///
    /// Returns error when receiving from the underlying channel fails
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

                    // If the receiver is gone, we should probably kill the read loop
                    // and exit
                    if t_events.send(Message::Error(InputError::from(e))).is_err() {
                        break;
                    };
                    continue;
                }
            };

            if r_kill.try_recv().is_ok() {
                break;
            }

            let input = Input::from_bytes(&buffer[..n]);

            // Same here. There is no point in reading if no one's receiving
            if t_events.send(Message::Input(input)).is_err() {
                break;
            }
        });

        (r_events, t_kill)
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        self.kill
            .send(())
            .expect("the receiver should not be dropped yet");
    }
}
