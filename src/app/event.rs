use std::collections::HashMap;

use edi::terminal::input::Input;

use edi::buffer::{self, Direction};

use super::Mode;

#[derive(Debug, Clone)]
pub enum Event {
    SwitchMode(Mode),
    InsertChar(char),
    DeleteChar,
    MoveCursor(buffer::Direction, usize),
    MoveHalfScreen(buffer::Direction),
    Quit,
    Submit,
    MoveToLineStart,
}

#[derive(Debug)]
pub struct InputMapper {
    mappings: HashMap<(Mode, Input), Event>,
}

impl Default for InputMapper {
    fn default() -> Self {
        let mut mapper = InputMapper {
            mappings: HashMap::new(),
        };

        mapper.add_default_mappings();
        mapper
    }
}

impl InputMapper {
    pub fn new() -> Self {
        Self::default()
    }

    fn add_default_mappings(&mut self) {
        self.add_mapping(
            Mode::Normal,
            Input::Control('d'),
            Event::MoveHalfScreen(Direction::Down),
        );
        self.add_mapping(
            Mode::Normal,
            Input::Control('u'),
            Event::MoveHalfScreen(Direction::Up),
        );
        self.add_mapping(
            Mode::Normal,
            Input::Keypress('h'),
            Event::MoveCursor(Direction::Left, 1),
        );
        self.add_mapping(
            Mode::Normal,
            Input::Keypress('j'),
            Event::MoveCursor(Direction::Down, 1),
        );
        self.add_mapping(
            Mode::Normal,
            Input::Keypress('k'),
            Event::MoveCursor(Direction::Up, 1),
        );
        self.add_mapping(
            Mode::Normal,
            Input::Keypress('l'),
            Event::MoveCursor(Direction::Right, 1),
        );
        self.add_mapping(
            Mode::Normal,
            Input::Keypress('i'),
            Event::SwitchMode(Mode::Insert),
        );
        self.add_mapping(
            Mode::Normal,
            Input::Keypress(':'),
            Event::SwitchMode(Mode::Terminal),
        );
        self.add_mapping(Mode::Normal, Input::Keypress('0'), Event::MoveToLineStart);

        self.add_mapping(Mode::Insert, Input::Escape, Event::SwitchMode(Mode::Normal));
        self.add_mapping(Mode::Insert, Input::Enter, Event::InsertChar('\n'));
        self.add_mapping(Mode::Insert, Input::Backspace, Event::DeleteChar);
        self.add_mapping(
            Mode::Insert,
            Input::ArrowLeft,
            Event::MoveCursor(Direction::Left, 1),
        );
        self.add_mapping(
            Mode::Insert,
            Input::ArrowDown,
            Event::MoveCursor(Direction::Down, 1),
        );
        self.add_mapping(
            Mode::Insert,
            Input::ArrowUp,
            Event::MoveCursor(Direction::Up, 1),
        );
        self.add_mapping(
            Mode::Insert,
            Input::ArrowRight,
            Event::MoveCursor(Direction::Right, 1),
        );

        // Terminal mode
        self.add_mapping(
            Mode::Terminal,
            Input::Escape,
            Event::SwitchMode(Mode::Normal),
        );
        self.add_mapping(Mode::Terminal, Input::Backspace, Event::DeleteChar);
        self.add_mapping(
            Mode::Terminal,
            Input::ArrowLeft,
            Event::MoveCursor(Direction::Left, 1),
        );
        self.add_mapping(
            Mode::Terminal,
            Input::ArrowRight,
            Event::MoveCursor(Direction::Right, 1),
        );
        self.add_mapping(Mode::Terminal, Input::Enter, Event::Submit);
    }

    pub fn add_mapping(&mut self, mode: Mode, input: Input, event: Event) {
        self.mappings.insert((mode, input), event);
    }

    pub fn map_input(&self, input: Input, mode: Mode) -> Option<Event> {
        if let Some(event) = self.mappings.get(&(mode, input.clone())) {
            return Some(event).cloned();
        }

        match (mode, input) {
            (Mode::Insert | Mode::Terminal, Input::Keypress(c)) => Some(Event::InsertChar(c)),
            _ => None,
        }
    }
}
