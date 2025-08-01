use std::io::Read;

use edi_term::input::Input;

use crate::event::{self, Event, Sender};

pub fn input_source(sender: Sender) {
    let mut buf = [0_u8; 4];
    let mut stdin = std::io::stdin().lock();

    let _span = edi_lib::span!("input_source");
    loop {
        let n = match stdin.read(&mut buf) {
            Ok(n) => n,
            Err(err) => {
                edi_lib::debug!("{err}");
                continue;
            }
        };

        let input = Input::from_bytes(&buf[..n]);
        let event = Event::new(event::Type::Input).with_payload(event::Payload::Input(input));

        if !sender.send_event(event) {
            break;
        };
    }
}
