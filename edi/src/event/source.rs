use std::thread::JoinHandle;

use super::sender::Sender;

pub struct SourcesHandle {
    senders: Vec<JoinHandle<()>>,
}

impl SourcesHandle {
    pub(super) fn new(capacity: usize) -> Self {
        Self {
            senders: Vec::with_capacity(capacity),
        }
    }

    pub(super) fn add(&mut self, handle: JoinHandle<()>) {
        self.senders.push(handle);
    }

    #[allow(unused)]
    pub fn join(self) {
        for sender in self.senders {
            let _ = sender.join();
        }
    }
}

pub trait Source: Send {
    fn run(&mut self, sender: Sender);
}

impl<F> Source for F
where
    F: for<'a> Fn(&'a Sender) + Send,
{
    fn run(&mut self, sender: Sender) {
        self(&sender);
    }
}
