use std::io::Read;

use edi_term::input::Input;

use crate::event::Sender;

pub fn input_source(sender: &Sender) {
    let _span = edi_lib::span!("input");

    let mut buf = [0_u8; 4];
    let mut stdin = std::io::stdin().lock();

    'outer: loop {
        let n = match stdin.read(&mut buf) {
            Ok(n) => {
                edi_lib::debug!("input: {:?}", &buf[..n]);
                n
            }
            Err(err) => {
                edi_lib::debug!("error: {err}");
                continue;
            }
        };

        let total_bytes = n;
        let mut chunk = &buf[..total_bytes];
        while !chunk.is_empty() {
            if chunk[0] != edi_term::input::ESCAPE || chunk.len() == 1 {
                let input = Input::from_bytes(&chunk[..1]);
                chunk = &chunk[1..];

                if !sender.send_input(input) {
                    break 'outer;
                }

                continue;
            }

            let input = Input::from_bytes(chunk);
            chunk = &[];

            edi_lib::debug!("got non-zero input: {input:?}");

            if !sender.send_input(input) {
                break 'outer;
            }
        }
    }
}
