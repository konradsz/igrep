#[mockall_double::double]
use super::result_list::ResultList;
#[mockall_double::double]
use crate::ig::Ig;
use anyhow::Result;
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent};
use std::time::Duration;

#[derive(Default)]
pub struct InputHandler {
    input_buffer: String,
    input_state: InputState,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InputState {
    Empty,
    Incomplete(String),
    Invalid(String),
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
        self.input_state = InputState::Empty;

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
            "g" => self.input_state = InputState::Incomplete("g…".into()),
            "d" => self.input_state = InputState::Incomplete("d…".into()),
            buf => {
                self.input_state = InputState::Invalid(buf.into());
                self.input_buffer.clear();
            }
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
            KeyCode::Down => result_list.next_match(),
            KeyCode::Up => result_list.previous_match(),
            KeyCode::Right => result_list.next_file(),
            KeyCode::PageDown => result_list.next_file(),
            KeyCode::Left => result_list.previous_file(),
            KeyCode::PageUp => result_list.previous_file(),
            KeyCode::Home => result_list.top(),
            KeyCode::End => result_list.bottom(),
            KeyCode::Delete => result_list.remove_current_entry(),
            KeyCode::Enter => ig.open_file(),
            KeyCode::F(5) => ig.search(result_list),
            KeyCode::Esc => {
                if matches!(self.input_state, InputState::Empty)
                    || matches!(self.input_state, InputState::Invalid(_))
                {
                    ig.exit();
                }
            }
            _ => (),
        }

        self.input_state = InputState::Empty;
    }

    pub fn get_state(&self) -> &InputState {
        &self.input_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ig::MockIg, ui::result_list::MockResultList};
    use test_case::test_case;

    #[test]
    fn down() {
        let mut input_handler = InputHandler::default();
        let mut result_list_mock = MockResultList::default();
        result_list_mock.expect_next_match().return_const(());
        input_handler.handle_non_char_input(
            KeyCode::Down,
            &mut result_list_mock,
            &mut MockIg::default(),
        );
    }

    #[test]
    fn up() {
        let mut input_handler = InputHandler::default();
        let mut result_list_mock = MockResultList::default();
        result_list_mock.expect_previous_match().return_const(());
        input_handler.handle_non_char_input(
            KeyCode::Up,
            &mut result_list_mock,
            &mut MockIg::default(),
        );
    }

    #[test]
    fn right() {
        let mut input_handler = InputHandler::default();
        let mut result_list_mock = MockResultList::default();
        result_list_mock.expect_next_file().return_const(());
        input_handler.handle_non_char_input(
            KeyCode::Right,
            &mut result_list_mock,
            &mut MockIg::default(),
        );
    }

    #[test]
    fn page_down() {
        let mut input_handler = InputHandler::default();
        let mut result_list_mock = MockResultList::default();
        result_list_mock.expect_next_file().return_const(());
        input_handler.handle_non_char_input(
            KeyCode::PageDown,
            &mut result_list_mock,
            &mut MockIg::default(),
        );
    }

    #[test]
    fn left() {
        let mut input_handler = InputHandler::default();
        let mut result_list_mock = MockResultList::default();
        result_list_mock.expect_previous_file().return_const(());
        input_handler.handle_non_char_input(
            KeyCode::Left,
            &mut result_list_mock,
            &mut MockIg::default(),
        );
    }

    #[test]
    fn page_up() {
        let mut input_handler = InputHandler::default();
        let mut result_list_mock = MockResultList::default();
        result_list_mock.expect_previous_file().return_const(());
        input_handler.handle_non_char_input(
            KeyCode::PageUp,
            &mut result_list_mock,
            &mut MockIg::default(),
        );
    }

    #[test]
    fn home() {
        let mut input_handler = InputHandler::default();
        let mut result_list_mock = MockResultList::default();
        result_list_mock.expect_top().return_const(());
        input_handler.handle_non_char_input(
            KeyCode::Home,
            &mut result_list_mock,
            &mut MockIg::default(),
        );
    }

    #[test]
    fn end() {
        let mut input_handler = InputHandler::default();
        let mut result_list_mock = MockResultList::default();
        result_list_mock.expect_bottom().return_const(());
        input_handler.handle_non_char_input(
            KeyCode::End,
            &mut result_list_mock,
            &mut MockIg::default(),
        );
    }

    #[test]
    fn delete() {
        let mut input_handler = InputHandler::default();
        let mut result_list_mock = MockResultList::default();
        result_list_mock
            .expect_remove_current_entry()
            .return_const(());
        input_handler.handle_non_char_input(
            KeyCode::Delete,
            &mut result_list_mock,
            &mut MockIg::default(),
        );
    }

    #[test]
    fn enter() {
        let mut input_handler = InputHandler::default();
        let mut ig_mock = MockIg::default();
        ig_mock.expect_open_file().return_const(());
        input_handler.handle_non_char_input(
            KeyCode::Enter,
            &mut MockResultList::default(),
            &mut ig_mock,
        );
    }

    #[test]
    fn f5() {
        let mut input_handler = InputHandler::default();
        let mut ig_mock = MockIg::default();
        ig_mock.expect_search().return_const(());
        input_handler.handle_non_char_input(
            KeyCode::F(5),
            &mut MockResultList::default(),
            &mut ig_mock,
        );
    }

    #[test_case(InputState::Empty)]
    #[test_case(InputState::Invalid("input".into()))]
    fn esc_to_exit(initial_state: InputState) {
        let mut input_handler = InputHandler::default();
        input_handler.input_state = initial_state;

        let mut ig_mock = MockIg::default();
        ig_mock.expect_exit().return_const(());
        input_handler.handle_non_char_input(
            KeyCode::Esc,
            &mut MockResultList::default(),
            &mut ig_mock,
        );
    }

    #[test]
    fn esc_to_clear() {
        let mut input_handler = InputHandler::default();
        input_handler.input_state = InputState::Incomplete("input".into());
        input_handler.handle_non_char_input(
            KeyCode::Esc,
            &mut MockResultList::default(),
            &mut MockIg::default(),
        );

        assert_eq!(input_handler.input_state, InputState::Empty);
    }
}
