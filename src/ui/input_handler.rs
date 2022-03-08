use anyhow::Result;
use std::time::Duration;

use crossterm::event::{poll, read, Event, KeyCode, KeyEvent};

use super::result_list::ResultList;
use crate::ig::Ig;

#[derive(Default)]
pub(crate) struct InputHandler {
    input_buffer: String,
}

impl InputHandler {
    pub(crate) fn handle_input(&mut self, result_list: &mut ResultList, ig: &mut Ig) -> Result<()> {
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
            KeyCode::Down => result_list.next_match(),
            KeyCode::Up => result_list.previous_match(),
            KeyCode::Right | KeyCode::PageDown => result_list.next_file(),
            KeyCode::Left | KeyCode::PageUp => result_list.previous_file(),
            KeyCode::Home => result_list.top(),
            KeyCode::End => result_list.bottom(),
            KeyCode::Delete => result_list.remove_current_entry(),
            KeyCode::Enter => ig.open_file(),
            KeyCode::F(5) => ig.search(result_list),
            KeyCode::Esc => ig.exit(),
            _ => (),
        }
    }
}
