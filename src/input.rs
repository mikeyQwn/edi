use std::{io::Read, os::fd::AsFd, sync::mpsc::Receiver, sync::mpsc::Sender};

#[derive(Clone, Copy, Debug)]
pub enum InputEvent {
    Keypress(char),
    Unimplemented([u8; 4], u8),
}

pub fn to_event_stream<H>(
    input_handle: H,
) -> (Receiver<InputEvent>, Receiver<std::io::Error>, Sender<()>)
where
    H: Read + AsFd + Send + 'static,
{
    let mut buffer = [0u8; 4];
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
                let _ = t_errors.send(e);
                continue;
            }
        };
        if r_kill.try_recv().is_ok() {
            break;
        }

        let event = match buffer[0] {
            c if c.is_ascii() => InputEvent::Keypress(c as char),
            _ => InputEvent::Unimplemented(buffer, n as u8),
        };

        t_events.send(event).unwrap();
    });

    (r_events, r_errors, t_kill)
}
