use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use edi::buffer;
use edi::string::position::{GlobalPosition, LinePosition};
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
    pub const fn move_once(action: MoveAction) -> Self {
        Self::Move { action, repeat: 1 }
    }
}

#[derive(Debug, Clone)]
pub enum MoveAction {
    Regular(Direction),
    HalfScreen(Direction),
    InLine(LinePosition),
    Global(GlobalPosition),
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
        self.add_default_mappings_n();
        self.add_default_mappings_i();
        self.add_default_mappings_t();
    }

    fn add_default_mappings_n(&mut self) {
        let mut map = |input, action| {
            self.add_mapping(Mode::Normal, input, action);
        };

        map(
            Input::Control('d'),
            Action::move_once(MoveAction::HalfScreen(Direction::Down)),
        );
        map(
            Input::Control('u'),
            Action::move_once(MoveAction::HalfScreen(Direction::Up)),
        );
        map(
            Input::Keypress('h'),
            Action::move_once(MoveAction::Regular(Direction::Left)),
        );
        map(
            Input::Keypress('j'),
            Action::move_once(MoveAction::Regular(Direction::Down)),
        );
        map(
            Input::Keypress('k'),
            Action::move_once(MoveAction::Regular(Direction::Up)),
        );
        map(
            Input::Keypress('l'),
            Action::move_once(MoveAction::Regular(Direction::Right)),
        );
        map(Input::Keypress('i'), Action::SwitchMode(Mode::Insert));
        map(Input::Keypress(':'), Action::SwitchMode(Mode::Terminal));
        map(
            Input::Keypress('0'),
            Action::move_once(MoveAction::InLine(LinePosition::Start)),
        );
        map(
            Input::Keypress('$'),
            Action::move_once(MoveAction::InLine(LinePosition::End)),
        );
        map(
            Input::Keypress('^'),
            Action::move_once(MoveAction::InLine(LinePosition::CharacterStart)),
        );
        map(
            Input::Keypress('G'),
            Action::move_once(MoveAction::Global(GlobalPosition::End)),
        );
    }

    fn add_default_mappings_i(&mut self) {
        let mut map = |input, action| {
            self.add_mapping(Mode::Insert, input, action);
        };

        map(Input::Escape, Action::SwitchMode(Mode::Normal));
        map(Input::Enter, Action::InsertChar('\n'));
        map(Input::Backspace, Action::DeleteChar);
        map(
            Input::ArrowLeft,
            Action::move_once(MoveAction::Regular(Direction::Left)),
        );
        map(
            Input::ArrowDown,
            Action::move_once(MoveAction::Regular(Direction::Down)),
        );
        map(
            Input::ArrowUp,
            Action::move_once(MoveAction::Regular(Direction::Up)),
        );
        map(
            Input::ArrowRight,
            Action::move_once(MoveAction::Regular(Direction::Right)),
        );
    }

    fn add_default_mappings_t(&mut self) {
        let mut map = |input, action| {
            self.add_mapping(Mode::Terminal, input, action);
        };

        map(Input::Escape, Action::SwitchMode(Mode::Normal));
        map(Input::Backspace, Action::DeleteChar);
        map(
            Input::ArrowLeft,
            Action::move_once(MoveAction::Regular(Direction::Left)),
        );
        map(
            Input::ArrowRight,
            Action::move_once(MoveAction::Regular(Direction::Left)),
        );
        map(Input::Enter, Action::Submit);
    }

    pub fn add_mapping(&mut self, mode: Mode, input: Input, action: Action) {
        self.mappings.insert((mode, input), action);
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
