use crate::input::Input;

use super::AppMode;

#[derive(Debug)]
pub enum Event {
    SwitchMode(AppMode),
    InsertChar(char),
    Quit,
}

pub fn map_input(input: &Input, mode: &AppMode) -> Option<Event> {
    match mode {
        AppMode::Normal => map_normal(input),
        AppMode::Insert => map_insert(input),
        AppMode::Terminal { .. } => map_terminal(input),
    }
}

fn map_normal(input: &Input) -> Option<Event> {
    match input {
        &Input::Escape => Some(Event::Quit),
        &Input::Keypress('i') => Some(Event::SwitchMode(AppMode::Insert)),
        _ => None,
    }
}

fn map_insert(input: &Input) -> Option<Event> {
    match input {
        &Input::Escape => Some(Event::SwitchMode(AppMode::Normal)),
        &Input::Keypress(c) => Some(Event::InsertChar(c)),
        _ => None,
    }
}

fn map_terminal(input: &Input) -> Option<Event> {
    match input {
        &Input::Escape => Some(Event::SwitchMode(AppMode::Normal)),
        _ => None,
    }
}
