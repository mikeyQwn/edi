use std::{collections::HashMap, sync::mpsc};

use edi_lib::brand::{Id, Tag};

use super::{
    sender::{EventBuffer, Sender},
    source::{Source, SourcesHandle},
    Event, Payload,
};

pub trait Handler<State> {
    fn handle(&mut self, state: &mut State, event: &Event, buf: &mut EventBuffer);
    fn interested_in(&self, own_id: Id, event: &Event) -> bool {
        let _ = (own_id, event);
        true
    }
}

pub struct EventManager<State> {
    tag: Tag,

    tx: mpsc::Sender<Payload>,
    rx: mpsc::Receiver<Payload>,

    attached_sources: Vec<Box<dyn Source>>,
    attached_handlers: HashMap<Id, Box<dyn Handler<State>>>,
    piped_events: Vec<Payload>,
}

impl<State> EventManager<State> {
    #[must_use]
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let tag = Tag::new();
        Self {
            tag,

            tx,
            rx,

            attached_sources: Vec::new(),
            attached_handlers: HashMap::new(),
            piped_events: Vec::new(),
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
        let child_id = self.tag.child_id();
        let _ = self.attached_handlers.insert(child_id, Box::new(handler));
    }

    pub fn pipe_event(&mut self, event: Payload) {
        self.piped_events.push(event);
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

        while let Some(payload) = self.piped_events.pop() {
            Self::handle_event(
                self.attached_handlers.iter_mut(),
                &mut state,
                &mut event_buffer,
                &Event::without_source(payload),
            );
        }

        'outer: loop {
            while let Some(event) = event_buffer.pop_first() {
                if event.payload.is_quit() {
                    break 'outer;
                }

                Self::handle_event(
                    self.attached_handlers.iter_mut(),
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
                    self.attached_handlers.iter_mut(),
                    &mut state,
                    &mut event_buffer,
                    &Event::without_source(event),
                );
            }
        }

        handle
    }

    fn handle_event<'a>(
        handlers: impl Iterator<Item = (&'a Id, &'a mut Box<dyn Handler<State>>)>,
        state: &'a mut State,
        buf: &'a mut EventBuffer,
        event: &'a Event,
    ) {
        for (&id, handler) in handlers {
            if !handler.interested_in(id, event) {
                continue;
            }

            handler.handle(state, event, buf.with_id(id));
        }
    }
}
