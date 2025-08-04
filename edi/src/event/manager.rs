use std::sync::mpsc;

use super::{
    sender::{EventBuffer, Sender},
    source::{Source, SourcesHandle},
    Event,
};

pub trait Handler<State> {
    fn handle(&mut self, state: &mut State, event: &Event, buf: &mut EventBuffer);
    fn interested_in(&self, event: &Event) -> bool {
        let _ = event;
        true
    }
}

pub struct EventManager<State> {
    tx: mpsc::Sender<Event>,
    rx: mpsc::Receiver<Event>,

    attached_sources: Vec<Box<dyn Source>>,
    attached_handlers: Vec<Box<dyn Handler<State>>>,
}

impl<State> EventManager<State> {
    #[must_use]
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            tx,
            rx,

            attached_sources: Vec::new(),
            attached_handlers: Vec::new(),
        }
    }

    fn new_sender(&self) -> Sender {
        Sender {
            tx: self.tx.clone(),
        }
    }

    pub fn attach_source<Src>(&mut self, source: Src)
    where
        Src: Source + Send + 'static,
    {
        self.attached_sources.push(Box::new(source));
    }

    pub fn attach_handler<Hnd>(&mut self, handler: Hnd)
    where
        Hnd: Handler<State> + 'static,
    {
        self.attached_handlers.push(Box::new(handler));
    }

    pub fn run(mut self, mut state: State) -> SourcesHandle {
        let mut handle = SourcesHandle::new(self.attached_sources.len());
        let sources = std::mem::take(&mut self.attached_sources);

        for mut source in sources {
            let sender = self.new_sender();
            handle.add(std::thread::spawn(move || {
                source.run(sender);
            }));
        }

        let mut event_buffer = EventBuffer::new();

        'outer: loop {
            while let Some(event) = event_buffer.pop_first() {
                if event.is_quit() {
                    break 'outer;
                }

                Self::handle_event(
                    &mut self.attached_handlers,
                    &mut state,
                    &mut event_buffer,
                    &event,
                );
            }

            if let Ok(event) = self.rx.recv() {
                if event.is_quit() {
                    break 'outer;
                }

                Self::handle_event(
                    &mut self.attached_handlers,
                    &mut state,
                    &mut event_buffer,
                    &event,
                );
            }
        }

        handle
    }

    fn handle_event(
        handlers: &mut [Box<dyn Handler<State>>],
        state: &mut State,
        buf: &mut EventBuffer,
        event: &Event,
    ) {
        for handler in handlers {
            if !handler.interested_in(event) {
                continue;
            }

            handler.handle(state, event, buf);
        }
    }
}
