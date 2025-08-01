pub mod handlers;
pub mod sources;

use std::{sync::mpsc, thread::JoinHandle};

use edi_term::input::Input;

pub struct SourcesHandle {
    senders: Vec<JoinHandle<()>>,
}

impl SourcesHandle {
    fn new(capacity: usize) -> Self {
        Self {
            senders: Vec::with_capacity(capacity),
        }
    }

    fn add(&mut self, handle: JoinHandle<()>) {
        self.senders.push(handle);
    }

    pub fn join(self) {
        for sender in self.senders {
            let _ = sender.join();
        }
    }
}

pub struct Sender {
    tx: mpsc::Sender<Event>,
}

impl Sender {
    pub fn send_event(&self, event: Event) -> bool {
        self.tx.send(event).is_ok()
    }
}

pub trait Source: Send {
    fn run(&mut self, sender: Sender);
}

impl<F> Source for F
where
    F: Fn(Sender) + Send,
{
    fn run(&mut self, sender: Sender) {
        self(sender)
    }
}

pub trait Handler {
    fn handle(&mut self, event: &Event, sender: &Sender);
    fn interested_in(&self, event: &Event) -> bool {
        let _ = event;
        true
    }
}

pub struct EventManager {
    tx: mpsc::Sender<Event>,
    rx: mpsc::Receiver<Event>,

    attached_sources: Vec<Box<dyn Source>>,
    attached_handlers: Vec<Box<dyn Handler>>,
}

impl EventManager {
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
        Hnd: Handler + 'static,
    {
        self.attached_handlers.push(Box::new(handler));
    }

    pub fn run(mut self) -> SourcesHandle {
        let mut handle = SourcesHandle::new(self.attached_sources.len());
        let sources = std::mem::take(&mut self.attached_sources);

        for mut source in sources {
            let sender = self.new_sender();
            handle.add(std::thread::spawn(move || {
                source.run(sender);
            }));
        }

        let sender = Sender { tx: self.tx };

        while let Ok(event) = self.rx.recv() {
            if event.is_quit() {
                break;
            }

            for handler in self.attached_handlers.iter_mut() {
                if !handler.interested_in(&event) {
                    continue;
                }

                handler.handle(&event, &sender);
            }
        }

        handle
    }
}

#[derive(Debug, Clone)]
struct Event {
    ty: Type,
    payload: Option<Payload>,
}

impl Event {
    #[must_use]
    pub fn new(ty: Type) -> Self {
        Self { ty, payload: None }
    }

    #[must_use]
    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn is_quit(&self) -> bool {
        self.ty == Type::Quit
    }
}

#[derive(Debug, Clone)]
enum Payload {
    Input(Input),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Type {
    Input,
    Quit,
}
