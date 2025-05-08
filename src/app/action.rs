use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use edi::buffer;
use edi::string::position::LinePosition;
use edi::terminal::input::Input;

use super::Mode;

#[derive(Debug, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl From<Direction> for buffer::Direction {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Up => Self::Up,
            Direction::Down => Self::Down,
            Direction::Left => Self::Left,
            Direction::Right => Self::Right,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    SwitchMode(Mode),
    InsertChar(char),
    DeleteChar,
    Quit,
    Submit,
    Move { action: MoveAction, repeat: usize },
}

impl Action {
    pub fn move_once(action: MoveAction) -> Self {
        Self::Move { action, repeat: 1 }
    }
}

#[derive(Debug, Clone)]
pub enum MoveAction {
    Regular(Direction),
    HalfScreen(Direction),
    To(LinePosition),
}

trait KeyPair<K1, K2> {
    fn key1(&self) -> &K1;
    fn key2(&self) -> &K2;
}

impl<K1, K2> KeyPair<K1, K2> for (K1, K2) {
    fn key1(&self) -> &K1 {
        &self.0
    }

    fn key2(&self) -> &K2 {
        &self.1
    }
}

impl<'a, K1, K2> KeyPair<K1, K2> for (&'a K1, &'a K2) {
    fn key1(&self) -> &K1 {
        self.0
    }

    fn key2(&self) -> &K2 {
        self.1
    }
}

impl<'a, K1, K2> Borrow<dyn KeyPair<K1, K2> + 'a> for (K1, K2)
where
    K1: Eq + Hash + 'a,
    K2: Eq + Hash + 'a,
{
    fn borrow(&self) -> &(dyn KeyPair<K1, K2> + 'a) {
        self
    }
}

impl<K1: Hash, K2: Hash> Hash for dyn KeyPair<K1, K2> + '_ {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key1().hash(state);
        self.key2().hash(state);
    }
}

impl<A: Eq, B: Eq> PartialEq for dyn KeyPair<A, B> + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.key1() == other.key1() && self.key2() == other.key2()
    }
}

impl<A: Eq, B: Eq> Eq for dyn KeyPair<A, B> + '_ {}

#[derive(Debug)]
pub struct InputMapper {
    mappings: HashMap<(Mode, Input), Action>,
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
    fn add_default_mappings(&mut self) {
        self.add_mapping(
            Mode::Normal,
            Input::Control('d'),
            Action::Move {
                action: MoveAction::HalfScreen(Direction::Down),
                repeat: 1,
            },
        );
        self.add_mapping(
            Mode::Normal,
            Input::Control('u'),
            Action::Move {
                action: MoveAction::HalfScreen(Direction::Up),
                repeat: 1,
            },
        );
        self.add_mapping(
            Mode::Normal,
            Input::Keypress('h'),
            Action::Move {
                action: MoveAction::Regular(Direction::Left),
                repeat: 1,
            },
        );
        self.add_mapping(
            Mode::Normal,
            Input::Keypress('j'),
            Action::Move {
                action: MoveAction::Regular(Direction::Down),
                repeat: 1,
            },
        );
        self.add_mapping(
            Mode::Normal,
            Input::Keypress('k'),
            Action::Move {
                action: MoveAction::Regular(Direction::Up),
                repeat: 1,
            },
        );
        self.add_mapping(
            Mode::Normal,
            Input::Keypress('l'),
            Action::Move {
                action: MoveAction::Regular(Direction::Right),
                repeat: 1,
            },
        );
        self.add_mapping(
            Mode::Normal,
            Input::Keypress('i'),
            Action::SwitchMode(Mode::Insert),
        );
        self.add_mapping(
            Mode::Normal,
            Input::Keypress(':'),
            Action::SwitchMode(Mode::Terminal),
        );
        self.add_mapping(
            Mode::Normal,
            Input::Keypress('0'),
            Action::Move {
                action: MoveAction::To(LinePosition::Start),
                repeat: 1,
            },
        );
        self.add_mapping(
            Mode::Normal,
            Input::Keypress('$'),
            Action::Move {
                action: MoveAction::To(LinePosition::End),
                repeat: 1,
            },
        );
        self.add_mapping(
            Mode::Normal,
            Input::Keypress('^'),
            Action::Move {
                action: MoveAction::To(LinePosition::CharacterStart),
                repeat: 1,
            },
        );

        self.add_mapping(
            Mode::Insert,
            Input::Escape,
            Action::SwitchMode(Mode::Normal),
        );
        self.add_mapping(Mode::Insert, Input::Enter, Action::InsertChar('\n'));
        self.add_mapping(Mode::Insert, Input::Backspace, Action::DeleteChar);
        self.add_mapping(
            Mode::Insert,
            Input::ArrowLeft,
            Action::move_once(MoveAction::Regular(Direction::Left)),
        );
        self.add_mapping(
            Mode::Insert,
            Input::ArrowDown,
            Action::move_once(MoveAction::Regular(Direction::Down)),
        );
        self.add_mapping(
            Mode::Insert,
            Input::ArrowUp,
            Action::move_once(MoveAction::Regular(Direction::Up)),
        );
        self.add_mapping(
            Mode::Insert,
            Input::ArrowRight,
            Action::move_once(MoveAction::Regular(Direction::Right)),
        );

        // Terminal mode
        self.add_mapping(
            Mode::Terminal,
            Input::Escape,
            Action::SwitchMode(Mode::Normal),
        );
        self.add_mapping(Mode::Terminal, Input::Backspace, Action::DeleteChar);
        self.add_mapping(
            Mode::Terminal,
            Input::ArrowLeft,
            Action::move_once(MoveAction::Regular(Direction::Left)),
        );
        self.add_mapping(
            Mode::Terminal,
            Input::ArrowRight,
            Action::move_once(MoveAction::Regular(Direction::Left)),
        );
        self.add_mapping(Mode::Terminal, Input::Enter, Action::Submit);
    }

    pub fn add_mapping(&mut self, mode: Mode, input: Input, event: Action) {
        self.mappings.insert((mode, input), event);
    }

    pub fn map_input(&self, input: &Input, mode: Mode) -> Option<Action> {
        if let Some(event) = self
            .mappings
            .get(&(&mode, input) as &dyn KeyPair<Mode, Input>)
        {
            return Some(event).cloned();
        }

        match (mode, input) {
            (Mode::Insert | Mode::Terminal, Input::Keypress(c)) => Some(Action::InsertChar(*c)),
            _ => None,
        }
    }
}
