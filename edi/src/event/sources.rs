use std::io::Read;

use edi_term::input::Input;

use crate::event::{self, Event, Sender};

pub fn input_source(sender: Sender) {
    let mut buf = [0_u8; 8];
    let mut stdin = std::io::stdin().lock();

    let _span = edi_lib::span!("input_source");

    'outer: loop {
        let n = match stdin.read(&mut buf[..]) {
            Ok(n) => n,
            Err(err) => {
                edi_lib::debug!("{err}");
                continue;
            }
        };

        let total_bytes = n;
        let mut chunk = &buf[..total_bytes];
        while !chunk.is_empty() {
            if chunk[0] != edi_term::input::ESCAPE || chunk.len() == 1 {
                let input = Input::from_bytes(&chunk[..1]);

                chunk = &chunk[1..];
                let event =
                    Event::new(event::Type::Input).with_payload(event::Payload::Input(input));
                if !sender.send_event(event) {
                    break 'outer;
                };

                continue;
            };

            let input = Input::from_bytes(&chunk[..]);
            let event = Event::new(event::Type::Input).with_payload(event::Payload::Input(input));

            if !sender.send_event(event) {
                break 'outer;
            };
        }
    }
}
