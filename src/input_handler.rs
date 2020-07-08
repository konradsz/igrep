use std::{collections::HashMap, error::Error, time::Duration};

use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};

use crate::ig::Ig;

type Operation = fn(&mut Ig);

#[derive(Default)]
pub struct InputHandler {
    operations: HashMap<KeyEvent, Operation>,
    previous_key: Option<KeyEvent>,
}

fn no_mod(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
    }
}

fn shift_mod(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::SHIFT,
    }
}

impl InputHandler {
    pub fn new() -> Self {
        let mut operations = HashMap::new();

        operations.insert(no_mod(KeyCode::Down), InputHandler::next_match as Operation);
        operations.insert(
            no_mod(KeyCode::Char('j')),
            InputHandler::next_match as Operation,
        );

        operations.insert(
            no_mod(KeyCode::Up),
            InputHandler::previous_match as Operation,
        );
        operations.insert(
            no_mod(KeyCode::Char('k')),
            InputHandler::previous_match as Operation,
        );

        operations.insert(no_mod(KeyCode::Right), InputHandler::next_file as Operation);
        operations.insert(
            no_mod(KeyCode::Char('l')),
            InputHandler::next_file as Operation,
        );

        operations.insert(
            no_mod(KeyCode::Left),
            InputHandler::previous_file as Operation,
        );
        operations.insert(
            no_mod(KeyCode::Char('h')),
            InputHandler::previous_file as Operation,
        );

        operations.insert(no_mod(KeyCode::Char('g')), InputHandler::top as Operation);

        operations.insert(
            shift_mod(KeyCode::Char('G')),
            InputHandler::bottom as Operation,
        );

        operations.insert(no_mod(KeyCode::Enter), InputHandler::open_file as Operation);

        operations.insert(
            no_mod(KeyCode::Esc),
            InputHandler::exit_iglication as Operation,
        );
        operations.insert(
            no_mod(KeyCode::Char('q')),
            InputHandler::exit_iglication as Operation,
        );

        Self {
            operations,
            previous_key: None,
        }
    }

    pub fn handle_input(&mut self, ig: &mut Ig) -> Result<(), Box<dyn Error>> {
        let poll_timeout = Duration::from_millis(if ig.is_idle() { 1_000 } else { 0 });
        if poll(poll_timeout)? {
            let read_event = read()?;
            if let Event::Key(key_event) = read_event {
                if let Some(operation) = self.operations.get(&key_event) {
                    operation(ig);
                }
                self.previous_key = Some(key_event);
            }
        }

        Ok(())
    }

    fn next_match(ig: &mut Ig) {
        ig.result_list.next_match();
    }

    fn previous_match(ig: &mut Ig) {
        ig.result_list.previous_match();
    }

    fn next_file(ig: &mut Ig) {
        ig.result_list.next_file();
    }

    fn previous_file(ig: &mut Ig) {
        ig.result_list.previous_file();
    }

    fn top(ig: &mut Ig) {
        ig.result_list.top();
    }

    fn bottom(ig: &mut Ig) {
        ig.result_list.bottom();
    }

    fn open_file(ig: &mut Ig) {
        ig.open_file();
    }

    fn exit_iglication(ig: &mut Ig) {
        ig.exit();
    }
}
