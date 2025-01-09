use crate::{buffer, input::Input};

use super::AppMode;

#[derive(Debug)]
pub enum Event {
    SwitchMode(AppMode),
    InsertChar(char),
    DeleteChar,
    MoveCursor(buffer::Direction, usize),
    MoveHalfScreen(buffer::Direction),
    Quit,
    Submit,
}

pub const fn map_input(input: &Input, mode: &AppMode) -> Option<Event> {
    match mode {
        AppMode::Normal => map_normal(input),
        AppMode::Insert => map_insert(input),
        AppMode::Terminal { .. } => map_terminal(input),
    }
}

const fn map_normal(input: &Input) -> Option<Event> {
    match *input {
        Input::Control('d') => Some(Event::MoveHalfScreen(buffer::Direction::Down)),
        Input::Control('u') => Some(Event::MoveHalfScreen(buffer::Direction::Up)),
        Input::Keypress('h') => Some(Event::MoveCursor(buffer::Direction::Left, 1)),
        Input::Keypress('j') => Some(Event::MoveCursor(buffer::Direction::Down, 1)),
        Input::Keypress('k') => Some(Event::MoveCursor(buffer::Direction::Up, 1)),
        Input::Keypress('l') => Some(Event::MoveCursor(buffer::Direction::Right, 1)),
        Input::Keypress('i') => Some(Event::SwitchMode(AppMode::Insert)),
        Input::Keypress(':') => Some(Event::SwitchMode(AppMode::Terminal)),
        _ => None,
    }
}

const fn map_insert(input: &Input) -> Option<Event> {
    match *input {
        Input::Escape => Some(Event::SwitchMode(AppMode::Normal)),
        Input::Keypress(c) => Some(Event::InsertChar(c)),
        Input::Enter => Some(Event::InsertChar('\n')),
        Input::Backspace => Some(Event::DeleteChar),
        Input::ArrowLeft => Some(Event::MoveCursor(buffer::Direction::Left, 1)),
        Input::ArrowDown => Some(Event::MoveCursor(buffer::Direction::Down, 1)),
        Input::ArrowUp => Some(Event::MoveCursor(buffer::Direction::Up, 1)),
        Input::ArrowRight => Some(Event::MoveCursor(buffer::Direction::Right, 1)),
        _ => None,
    }
}

const fn map_terminal(input: &Input) -> Option<Event> {
    match *input {
        Input::Escape => Some(Event::SwitchMode(AppMode::Normal)),
        Input::Keypress(c) => Some(Event::InsertChar(c)),
        Input::Backspace => Some(Event::DeleteChar),
        Input::ArrowLeft => Some(Event::MoveCursor(buffer::Direction::Left, 1)),
        Input::ArrowRight => Some(Event::MoveCursor(buffer::Direction::Right, 1)),
        Input::Enter => Some(Event::Submit),
        _ => None,
    }
}
