use std::{error::Error, time::Duration};

use crossterm::event::{poll, read, Event, KeyCode, KeyEvent};

use crate::ig::Ig;

#[derive(Default)]
pub struct InputHandler {
    input_buffer: String,
}

impl InputHandler {
    pub fn handle_input(&mut self, ig: &mut Ig) -> Result<(), Box<dyn Error>> {
        let poll_timeout = Duration::from_millis(1);
        if poll(poll_timeout)? {
            let read_event = read()?;
            if let Event::Key(key_event) = read_event {
                if matches!(key_event, KeyEvent {
                    code: KeyCode::Char(_),
                ..})
                {
                    self.handle_char_input(key_event.code, ig);
                } else {
                    self.handle_non_char_input(key_event.code, ig);
                }
            }
        }

        Ok(())
    }

    fn handle_char_input(&mut self, key_code: KeyCode, ig: &mut Ig) {
        if let KeyCode::Char(c) = key_code {
            self.input_buffer.push(c);
        }

        let consume_buffer_and_execute = |buffer: &mut String, op: &mut dyn FnMut()| {
            buffer.clear();
            op();
        };

        match self.input_buffer.as_str() {
            "j" => consume_buffer_and_execute(&mut self.input_buffer, &mut || {
                ig.result_list.next_match()
            }),
            "k" => consume_buffer_and_execute(&mut self.input_buffer, &mut || {
                ig.result_list.previous_match()
            }),
            "l" => consume_buffer_and_execute(&mut self.input_buffer, &mut || {
                ig.result_list.next_file()
            }),
            "h" => consume_buffer_and_execute(&mut self.input_buffer, &mut || {
                ig.result_list.previous_file()
            }),
            "gg" => {
                consume_buffer_and_execute(&mut self.input_buffer, &mut || ig.result_list.top())
            }
            "G" => {
                consume_buffer_and_execute(&mut self.input_buffer, &mut || ig.result_list.bottom())
            }
            "dd" => consume_buffer_and_execute(&mut self.input_buffer, &mut || {
                ig.result_list.remove_current_entry()
            }),
            "dw" => consume_buffer_and_execute(&mut self.input_buffer, &mut || {
                ig.result_list.remove_current_file()
            }),
            "q" => consume_buffer_and_execute(&mut self.input_buffer, &mut || ig.exit()),
            "g" | "d" => (),
            _ => self.input_buffer.clear(),
        }
    }

    fn handle_non_char_input(&mut self, key_code: KeyCode, ig: &mut Ig) {
        self.input_buffer.clear();

        match key_code {
            KeyCode::Down => ig.result_list.next_match(),
            KeyCode::Up => ig.result_list.previous_match(),
            KeyCode::Right | KeyCode::PageDown => ig.result_list.next_file(),
            KeyCode::Left | KeyCode::PageUp => ig.result_list.previous_file(),
            KeyCode::Home => ig.result_list.top(),
            KeyCode::End => ig.result_list.bottom(),
            KeyCode::Delete => ig.result_list.remove_current_entry(),
            KeyCode::Enter => ig.open_file(),
            KeyCode::F(5) => ig.search(),
            KeyCode::Esc => ig.exit(),
            _ => (),
        }
    }
}
