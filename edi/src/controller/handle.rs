use super::Controller;

// A handle to controller which allows to send queries and emit events
pub struct Handle<'a, State> {
    inner: &'a mut Controller<State>,
}

impl<'a, State> Handle<'a, State> {
    pub(super) fn new(inner: &'a mut Controller<State>) -> Self {
        Self { inner }
    }
}
