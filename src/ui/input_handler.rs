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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputState {
    Valid,
    Incomplete(String),
    Invalid(String),
}

impl Default for InputState {
    fn default() -> Self {
        Self::Valid
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
        self.input_state = InputState::Valid;

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
            KeyCode::Right | KeyCode::PageDown => result_list.next_file(),
            KeyCode::Left | KeyCode::PageUp => result_list.previous_file(),
            KeyCode::Home => result_list.top(),
            KeyCode::End => result_list.bottom(),
            KeyCode::Delete => result_list.remove_current_entry(),
            KeyCode::Enter => ig.open_file(),
            KeyCode::F(5) => ig.search(result_list),
            KeyCode::Esc => {
                if matches!(self.input_state, InputState::Valid)
                    || matches!(self.input_state, InputState::Invalid(_))
                {
                    ig.exit();
                }
            }
            _ => (),
        }

        self.input_state = InputState::Valid;
    }

    pub fn get_state(&self) -> &InputState {
        &self.input_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ig::MockIg, ui::result_list::MockResultList};
    use crossterm::event::KeyCode::{Char, Esc};
    use test_case::test_case;

    fn handle_key(key_code: KeyCode, result_list: &mut MockResultList, ig: &mut MockIg) {
        let mut input_handler = InputHandler::default();
        handle(&mut input_handler, key_code, result_list, ig);
    }

    fn handle_key_series(key_codes: &[KeyCode], result_list: &mut MockResultList, ig: &mut MockIg) {
        let mut input_handler = InputHandler::default();
        for key_code in key_codes {
            handle(&mut input_handler, *key_code, result_list, ig);
        }
    }

    fn handle(
        input_handler: &mut InputHandler,
        key_code: KeyCode,
        result_list: &mut MockResultList,
        ig: &mut MockIg,
    ) {
        match key_code {
            Char(character) => input_handler.handle_char_input(character, result_list, ig),
            _ => input_handler.handle_non_char_input(key_code, result_list, ig),
        }
    }

    #[test_case(KeyCode::Down; "down")]
    #[test_case(Char('j'); "j")]
    fn next_match(key_code: KeyCode) {
        let mut result_list_mock = MockResultList::default();
        result_list_mock
            .expect_next_match()
            .times(1)
            .return_const(());
        handle_key(key_code, &mut result_list_mock, &mut MockIg::default());
    }

    #[test_case(KeyCode::Up; "up")]
    #[test_case(Char('k'); "k")]
    fn previous_match(key_code: KeyCode) {
        let mut result_list_mock = MockResultList::default();
        result_list_mock
            .expect_previous_match()
            .times(1)
            .return_const(());
        handle_key(key_code, &mut result_list_mock, &mut MockIg::default());
    }

    #[test_case(KeyCode::Right; "right")]
    #[test_case(KeyCode::PageDown; "page down")]
    #[test_case(Char('l'); "l")]
    fn next_file(key_code: KeyCode) {
        let mut result_list_mock = MockResultList::default();
        result_list_mock
            .expect_next_file()
            .times(1)
            .return_const(());
        handle_key(key_code, &mut result_list_mock, &mut MockIg::default());
    }

    #[test_case(KeyCode::Left; "left")]
    #[test_case(KeyCode::PageUp; "page up")]
    #[test_case(Char('h'); "h")]
    fn previous_file(key_code: KeyCode) {
        let mut result_list_mock = MockResultList::default();
        result_list_mock
            .expect_previous_file()
            .times(1)
            .return_const(());
        handle_key(key_code, &mut result_list_mock, &mut MockIg::default());
    }

    #[test_case(&[KeyCode::Home]; "home")]
    #[test_case(&[Char('g'), Char('g')]; "gg")]
    fn top(key_codes: &[KeyCode]) {
        let mut result_list_mock = MockResultList::default();
        result_list_mock.expect_top().times(1).return_const(());
        handle_key_series(key_codes, &mut result_list_mock, &mut MockIg::default());
    }

    #[test_case(KeyCode::End; "end")]
    #[test_case(Char('G'); "G")]
    fn bottom(key_code: KeyCode) {
        let mut result_list_mock = MockResultList::default();
        result_list_mock.expect_bottom().times(1).return_const(());
        handle_key(key_code, &mut result_list_mock, &mut MockIg::default());
    }

    #[test_case(&[KeyCode::Delete]; "delete")]
    #[test_case(&[Char('d'), Char('d')]; "dd")]
    #[test_case(&[Char('g'), Char('d'), Char('w'), Char('d'), Char('d')]; "gdwdd")]
    fn remove_current_entry(key_codes: &[KeyCode]) {
        let mut result_list_mock = MockResultList::default();
        result_list_mock
            .expect_remove_current_entry()
            .times(1)
            .return_const(());
        handle_key_series(key_codes, &mut result_list_mock, &mut MockIg::default());
    }

    #[test_case(&[Char('d'), Char('w')]; "dw")]
    #[test_case(&[Char('w'), Char('d'), Char('w')]; "wdw")]
    fn remove_current_file(key_codes: &[KeyCode]) {
        let mut result_list_mock = MockResultList::default();
        result_list_mock
            .expect_remove_current_file()
            .times(1)
            .return_const(());
        handle_key_series(key_codes, &mut result_list_mock, &mut MockIg::default());
    }

    #[test]
    fn open_file() {
        let mut ig_mock = MockIg::default();
        ig_mock.expect_open_file().times(1).return_const(());
        handle_key(KeyCode::Enter, &mut MockResultList::default(), &mut ig_mock);
    }

    #[test]
    fn search() {
        let mut ig_mock = MockIg::default();
        ig_mock.expect_search().times(1).return_const(());
        handle_key(KeyCode::F(5), &mut MockResultList::default(), &mut ig_mock);
    }

    #[test_case(&[Char('q')]; "q")]
    #[test_case(&[Esc]; "empty input state")]
    #[test_case(&[Char('a'), Char('b'), Esc]; "invalid input state")]
    #[test_case(&[Char('d'), Esc, Esc]; "clear incomplete state first")]
    fn exit(key_codes: &[KeyCode]) {
        let mut ig_mock = MockIg::default();
        ig_mock.expect_exit().times(1).return_const(());
        handle_key_series(key_codes, &mut MockResultList::default(), &mut ig_mock);
    }
}
