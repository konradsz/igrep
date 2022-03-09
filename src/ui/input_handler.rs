#[mockall_double::double]
use super::result_list::ResultList;
#[mockall_double::double]
use crate::ig::Ig;
use anyhow::Result;
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent};
use std::time::Duration;

#[derive(Default)]
pub struct InputHandler {
    input_buffer: String, // TODO: remove, input_state can replace it
    input_state: InputState,
}

#[derive(Debug, PartialEq)]
pub enum InputState {
    Empty,
    Incomplete(String),
    Successful(String),
    Unsuccessful(String),
}

impl Default for InputState {
    fn default() -> Self {
        Self::Empty
    }
}

impl InputHandler {
    pub fn handle_input(&mut self, result_list: &mut ResultList, ig: &mut Ig) -> Result<()> {
        let poll_timeout = if ig.is_searching() {
            Duration::from_millis(1)
        } else {
            Duration::from_millis(100)
        };
        if poll(poll_timeout)? {
            let read_event = read()?;
            if let Event::Key(key_event) = read_event {
                match key_event {
                    KeyEvent {
                        code: KeyCode::Char(character),
                        ..
                    } => self.handle_char_input(character, result_list, ig),
                    _ => self.handle_non_char_input(key_event.code, result_list, ig),
                }
            }
        }

        Ok(())
    }

    fn handle_char_input(&mut self, character: char, result_list: &mut ResultList, ig: &mut Ig) {
        self.input_buffer.push(character);

        let consume_buffer_and_execute = |buffer: &mut String, op: &mut dyn FnMut()| {
            buffer.clear();
            op();
        };

        match self.input_buffer.as_str() {
            "j" => {
                consume_buffer_and_execute(&mut self.input_buffer, &mut || result_list.next_match())
            }
            "k" => consume_buffer_and_execute(&mut self.input_buffer, &mut || {
                result_list.previous_match()
            }),
            "l" => {
                consume_buffer_and_execute(&mut self.input_buffer, &mut || result_list.next_file())
            }
            "h" => consume_buffer_and_execute(&mut self.input_buffer, &mut || {
                result_list.previous_file()
            }),
            "gg" => consume_buffer_and_execute(&mut self.input_buffer, &mut || result_list.top()),
            "G" => consume_buffer_and_execute(&mut self.input_buffer, &mut || result_list.bottom()),
            "dd" => consume_buffer_and_execute(&mut self.input_buffer, &mut || {
                result_list.remove_current_entry()
            }),
            "dw" => consume_buffer_and_execute(&mut self.input_buffer, &mut || {
                result_list.remove_current_file()
            }),
            "q" => consume_buffer_and_execute(&mut self.input_buffer, &mut || ig.exit()),
            "g" | "d" => (),
            _ => self.input_buffer.clear(),
        }
    }

    fn handle_non_char_input(
        &mut self,
        key_code: KeyCode,
        result_list: &mut ResultList,
        ig: &mut Ig,
    ) {
        self.input_buffer.clear();

        match key_code {
            KeyCode::Down => {
                self.input_state = InputState::Successful("↓".into());
                result_list.next_match();
            }
            KeyCode::Up => {
                self.input_state = InputState::Successful("↑".into());
                result_list.previous_match();
            }
            KeyCode::Right => {
                self.input_state = InputState::Successful("→".into());
                result_list.next_file();
            }
            KeyCode::PageDown => {
                self.input_state = InputState::Successful("PgDn".into());
                result_list.next_file();
            }
            KeyCode::Left => {
                self.input_state = InputState::Successful("←".into());
                result_list.previous_file();
            }
            KeyCode::PageUp => {
                self.input_state = InputState::Successful("PgUp".into());
                result_list.previous_file();
            }
            KeyCode::Home => {
                self.input_state = InputState::Successful("Home".into());
                result_list.top();
            }
            KeyCode::End => {
                self.input_state = InputState::Successful("End".into());
                result_list.bottom();
            }
            KeyCode::Delete => {
                self.input_state = InputState::Successful("Del".into());
                result_list.remove_current_entry();
            }
            KeyCode::Enter => {
                self.input_state = InputState::Empty;
                ig.open_file();
            }
            KeyCode::F(5) => {
                self.input_state = InputState::Empty;
                ig.search(result_list);
            }
            KeyCode::Esc => ig.exit(),
            _ => (),
        }
    }

    pub fn get_state(&self) -> &InputState {
        &self.input_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ig::MockIg, ui::result_list::MockResultList};

    #[test]
    fn enter() {
        let mut input_handler = InputHandler::default();
        input_handler.input_state = InputState::Successful("input".into());

        let mut ig_mock = MockIg::default();
        ig_mock.expect_open_file().return_const(());
        input_handler.handle_non_char_input(
            KeyCode::Enter,
            &mut MockResultList::default(),
            &mut ig_mock,
        );

        assert_eq!(input_handler.input_state, InputState::Empty);
    }
}
