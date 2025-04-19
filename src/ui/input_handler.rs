use anyhow::Result;
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::Duration;

use crate::app::Application;

#[derive(Default)]
pub struct InputHandler {
    input_buffer: String,
    input_state: InputState,
    input_mode: InputMode,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum InputState {
    #[default]
    Valid,
    Incomplete(String),
    Invalid(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Normal,
    TextInsertion,
    Keymap,
}

impl InputHandler {
    pub fn handle_input<A: Application>(&mut self, app: &mut A) -> Result<()> {
        let poll_timeout = if app.is_searching() {
            Duration::from_millis(1)
        } else {
            Duration::from_millis(100)
        };

        if poll(poll_timeout)? {
            let read_event = read()?;
            if let Event::Key(key_event) = read_event {
                // The following line needs to be amended if and when enabling the
                // `KeyboardEnhancementFlags::REPORT_EVENT_TYPES` flag on unix.
                let event_kind_enabled = cfg!(target_family = "windows");
                let process_event = !event_kind_enabled || key_event.kind != KeyEventKind::Release;

                if process_event {
                    match self.input_mode {
                        InputMode::Normal => self.handle_key_in_normal_mode(key_event, app),
                        InputMode::TextInsertion => {
                            self.handle_key_in_text_insertion_mode(key_event, app)
                        }
                        InputMode::Keymap => self.handle_key_in_keymap_mode(key_event, app),
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_key_in_normal_mode<A: Application>(&mut self, key_event: KeyEvent, app: &mut A) {
        match key_event {
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => app.on_exit(),
            KeyEvent {
                code: KeyCode::Char(character),
                ..
            } => self.handle_char_input(character, app),
            _ => self.handle_non_char_input(key_event.code, app),
        }
    }

    fn handle_key_in_text_insertion_mode<A: Application>(
        &mut self,
        key_event: KeyEvent,
        app: &mut A,
    ) {
        match key_event {
            KeyEvent {
                code: KeyCode::Esc, ..
            }
            | KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }
            | KeyEvent {
                code: KeyCode::F(5),
                ..
            } => {
                self.input_mode = InputMode::Normal;
                app.on_toggle_popup();
            }
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: modifier,
                ..
            } => {
                if modifier == KeyModifiers::SHIFT {
                    app.on_char_inserted(c.to_ascii_uppercase());
                } else if modifier == KeyModifiers::NONE {
                    app.on_char_inserted(c);
                }
            }
            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => app.on_char_removed(),
            KeyEvent {
                code: KeyCode::Delete,
                ..
            } => app.on_char_deleted(),
            KeyEvent {
                code: KeyCode::Left,
                ..
            } => app.on_char_left(),
            KeyEvent {
                code: KeyCode::Right,
                ..
            } => app.on_char_right(),
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => {
                self.input_mode = InputMode::Normal;
                app.on_search();
                app.on_toggle_popup();
            }
            _ => (),
        }
    }

    fn handle_key_in_keymap_mode<A: Application>(&mut self, key_event: KeyEvent, app: &mut A) {
        match key_event {
            KeyEvent {
                code: KeyCode::Up, ..
            }
            | KeyEvent {
                code: KeyCode::Char('k'),
                ..
            } => app.on_keymap_up(),
            KeyEvent {
                code: KeyCode::Down,
                ..
            }
            | KeyEvent {
                code: KeyCode::Char('j'),
                ..
            } => app.on_keymap_down(),
            KeyEvent {
                code: KeyCode::Left,
                ..
            }
            | KeyEvent {
                code: KeyCode::Char('h'),
                ..
            } => app.on_keymap_left(),
            KeyEvent {
                code: KeyCode::Right,
                ..
            }
            | KeyEvent {
                code: KeyCode::Char('l'),
                ..
            } => app.on_keymap_right(),
            _ => {
                self.input_mode = InputMode::Normal;
                app.on_toggle_keymap();
            }
        }
    }

    fn handle_char_input<A: Application>(&mut self, character: char, app: &mut A) {
        self.input_buffer.push(character);
        self.input_state = InputState::Valid;

        let consume_buffer_and_execute = |buffer: &mut String, op: &mut dyn FnMut()| {
            buffer.clear();
            op();
        };

        match self.input_buffer.as_str() {
            // navigation
            "j" => consume_buffer_and_execute(&mut self.input_buffer, &mut || app.on_next_match()),
            "k" => {
                consume_buffer_and_execute(&mut self.input_buffer, &mut || app.on_previous_match())
            }
            "l" => consume_buffer_and_execute(&mut self.input_buffer, &mut || app.on_next_file()),
            "h" => {
                consume_buffer_and_execute(&mut self.input_buffer, &mut || app.on_previous_file())
            }
            "gg" => consume_buffer_and_execute(&mut self.input_buffer, &mut || app.on_top()),
            "G" => consume_buffer_and_execute(&mut self.input_buffer, &mut || app.on_bottom()),
            // deletion
            "dd" => consume_buffer_and_execute(&mut self.input_buffer, &mut || {
                app.on_remove_current_entry()
            }),
            "dw" => consume_buffer_and_execute(&mut self.input_buffer, &mut || {
                app.on_remove_current_file()
            }),
            // viewer
            "v" => consume_buffer_and_execute(&mut self.input_buffer, &mut || {
                app.on_toggle_context_viewer_vertical()
            }),
            "s" => consume_buffer_and_execute(&mut self.input_buffer, &mut || {
                app.on_toggle_context_viewer_horizontal()
            }),
            "+" => consume_buffer_and_execute(&mut self.input_buffer, &mut || {
                app.on_increase_context_viewer_size()
            }),
            "-" => consume_buffer_and_execute(&mut self.input_buffer, &mut || {
                app.on_decrease_context_viewer_size()
            }),
            // sort
            "n" => consume_buffer_and_execute(&mut self.input_buffer, &mut || {
                app.on_toggle_sort_name()
            }),
            "m" => consume_buffer_and_execute(&mut self.input_buffer, &mut || {
                app.on_toggle_sort_mtime()
            }),
            "a" => consume_buffer_and_execute(&mut self.input_buffer, &mut || {
                app.on_toggle_sort_atime()
            }),
            "c" => consume_buffer_and_execute(&mut self.input_buffer, &mut || {
                app.on_toggle_sort_ctime()
            }),
            // misc
            "q" => consume_buffer_and_execute(&mut self.input_buffer, &mut || app.on_exit()),
            "?" => {
                consume_buffer_and_execute(&mut self.input_buffer, &mut || app.on_toggle_keymap())
            }
            "/" => {
                self.input_mode = InputMode::TextInsertion;
                consume_buffer_and_execute(&mut self.input_buffer, &mut || app.on_toggle_popup())
            }
            // buffer for multikey inuts
            "g" => self.input_state = InputState::Incomplete("g…".into()),
            "d" => self.input_state = InputState::Incomplete("d…".into()),
            buf => {
                self.input_state = InputState::Invalid(buf.into());
                self.input_buffer.clear();
            }
        }
    }

    fn handle_non_char_input<A: Application>(&mut self, key_code: KeyCode, app: &mut A) {
        self.input_buffer.clear();

        match key_code {
            KeyCode::Down => app.on_next_match(),
            KeyCode::Up => app.on_previous_match(),
            KeyCode::Right | KeyCode::PageDown => app.on_next_file(),
            KeyCode::Left | KeyCode::PageUp => app.on_previous_file(),
            KeyCode::Home => app.on_top(),
            KeyCode::End => app.on_bottom(),
            KeyCode::Delete => app.on_remove_current_entry(),
            KeyCode::Enter => app.on_open_file(),
            KeyCode::F(1) => {
                self.input_mode = InputMode::Keymap;
                app.on_toggle_keymap();
            }
            KeyCode::F(5) => {
                self.input_mode = InputMode::TextInsertion;
                app.on_toggle_popup();
            }
            KeyCode::Esc => {
                if matches!(self.input_state, InputState::Valid)
                    || matches!(self.input_state, InputState::Invalid(_))
                {
                    app.on_exit();
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
    use crate::app::MockApplication;

    use super::*;
    use crossterm::event::KeyCode::{Char, Esc};
    use test_case::test_case;

    fn handle_key<A: Application>(key_code: KeyCode, app: &mut A) {
        let mut input_handler = InputHandler::default();
        handle(&mut input_handler, key_code, app);
    }

    fn handle_key_series<A: Application>(key_codes: &[KeyCode], app: &mut A) {
        let mut input_handler = InputHandler::default();
        for key_code in key_codes {
            handle(&mut input_handler, *key_code, app);
        }
    }

    fn handle<A: Application>(input_handler: &mut InputHandler, key_code: KeyCode, app: &mut A) {
        match key_code {
            Char(character) => input_handler.handle_char_input(character, app),
            _ => input_handler.handle_non_char_input(key_code, app),
        }
    }

    fn handle_key_keymap_mode<A: Application>(key_event: KeyEvent, app: &mut A) {
        let mut input_handler = InputHandler {
            input_mode: InputMode::Keymap,
            ..Default::default()
        };
        input_handler.handle_key_in_keymap_mode(key_event, app);
    }

    #[test_case(KeyCode::Down; "down")]
    #[test_case(Char('j'); "j")]
    fn next_match(key_code: KeyCode) {
        let mut app_mock = MockApplication::default();
        app_mock.expect_on_next_match().once().return_const(());
        handle_key(key_code, &mut app_mock);
    }

    #[test_case(KeyCode::Up; "up")]
    #[test_case(Char('k'); "k")]
    fn previous_match(key_code: KeyCode) {
        let mut app_mock = MockApplication::default();
        app_mock.expect_on_previous_match().once().return_const(());
        handle_key(key_code, &mut app_mock);
    }

    #[test_case(KeyCode::Right; "right")]
    #[test_case(KeyCode::PageDown; "page down")]
    #[test_case(Char('l'); "l")]
    fn next_file(key_code: KeyCode) {
        let mut app_mock = MockApplication::default();
        app_mock.expect_on_next_file().once().return_const(());
        handle_key(key_code, &mut app_mock);
    }

    #[test_case(KeyCode::Left; "left")]
    #[test_case(KeyCode::PageUp; "page up")]
    #[test_case(Char('h'); "h")]
    fn previous_file(key_code: KeyCode) {
        let mut app_mock = MockApplication::default();
        app_mock.expect_on_previous_file().once().return_const(());
        handle_key(key_code, &mut app_mock);
    }

    #[test_case(&[KeyCode::Home]; "home")]
    #[test_case(&[Char('g'), Char('g')]; "gg")]
    fn top(key_codes: &[KeyCode]) {
        let mut app_mock = MockApplication::default();
        app_mock.expect_on_top().once().return_const(());
        handle_key_series(key_codes, &mut app_mock);
    }

    #[test_case(KeyCode::End; "end")]
    #[test_case(Char('G'); "G")]
    fn bottom(key_code: KeyCode) {
        let mut app_mock = MockApplication::default();
        app_mock.expect_on_bottom().once().return_const(());
        handle_key(key_code, &mut app_mock);
    }

    #[test_case(&[KeyCode::Delete]; "delete")]
    #[test_case(&[Char('d'), Char('d')]; "dd")]
    #[test_case(&[Char('g'), Char('d'), Char('w'), Char('d'), Char('d')]; "gdwdd")]
    fn remove_current_entry(key_codes: &[KeyCode]) {
        let mut app_mock = MockApplication::default();
        app_mock
            .expect_on_remove_current_entry()
            .once()
            .return_const(());
        handle_key_series(key_codes, &mut app_mock);
    }

    #[test_case(&[Char('d'), Char('w')]; "dw")]
    #[test_case(&[Char('w'), Char('d'), Char('w')]; "wdw")]
    fn remove_current_file(key_codes: &[KeyCode]) {
        let mut app_mock = MockApplication::default();
        app_mock
            .expect_on_remove_current_file()
            .once()
            .return_const(());
        handle_key_series(key_codes, &mut app_mock);
    }

    #[test]
    fn toggle_vertical_context_viewer() {
        let mut app_mock = MockApplication::default();
        app_mock
            .expect_on_toggle_context_viewer_vertical()
            .once()
            .return_const(());
        handle_key(KeyCode::Char('v'), &mut app_mock);
    }

    #[test]
    fn toggle_horizontal_context_viewer() {
        let mut app_mock = MockApplication::default();
        app_mock
            .expect_on_toggle_context_viewer_horizontal()
            .once()
            .return_const(());
        handle_key(KeyCode::Char('s'), &mut app_mock);
    }

    #[test]
    fn open_file() {
        let mut app_mock = MockApplication::default();
        app_mock.expect_on_open_file().once().return_const(());
        handle_key(KeyCode::Enter, &mut app_mock);
    }

    #[test_case(KeyCode::F(5))]
    #[test_case(KeyCode::Char('/'))]
    fn search(key_code: KeyCode) {
        let mut app_mock = MockApplication::default();
        app_mock.expect_on_toggle_popup().once().return_const(());
        handle_key(key_code, &mut app_mock);
    }

    #[test_case(KeyCode::F(1))]
    #[test_case(KeyCode::Char('?'))]
    fn keymap_open(key_code: KeyCode) {
        let mut app_mock = MockApplication::default();
        app_mock.expect_on_toggle_keymap().once().return_const(());
        handle_key(key_code, &mut app_mock);
    }

    #[test_case(KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE))]
    #[test_case(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE))]
    #[test_case(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))]
    #[test_case(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))]
    #[test_case(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL))]
    fn keymap_close(event: KeyEvent) {
        let mut app_mock = MockApplication::default();
        app_mock.expect_on_toggle_keymap().once().return_const(());
        handle_key_keymap_mode(event, &mut app_mock);
    }

    #[test_case(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE))]
    #[test_case(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE))]
    fn keymap_up(event: KeyEvent) {
        let mut app_mock = MockApplication::default();
        app_mock.expect_on_keymap_up().once().return_const(());
        handle_key_keymap_mode(event, &mut app_mock);
    }

    #[test_case(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE))]
    #[test_case(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE))]
    fn keymap_down(event: KeyEvent) {
        let mut app_mock = MockApplication::default();
        app_mock.expect_on_keymap_down().once().return_const(());
        handle_key_keymap_mode(event, &mut app_mock);
    }

    #[test_case(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE))]
    #[test_case(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE))]
    fn keymap_left(event: KeyEvent) {
        let mut app_mock = MockApplication::default();
        app_mock.expect_on_keymap_left().once().return_const(());
        handle_key_keymap_mode(event, &mut app_mock);
    }

    #[test_case(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE))]
    #[test_case(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE))]
    fn keymap_right(event: KeyEvent) {
        let mut app_mock = MockApplication::default();
        app_mock.expect_on_keymap_right().once().return_const(());
        handle_key_keymap_mode(event, &mut app_mock);
    }

    #[test_case(&[Char('q')]; "q")]
    #[test_case(&[Esc]; "empty input state")]
    #[test_case(&[Char('a'), Char('b'), Esc]; "invalid input state")]
    #[test_case(&[Char('d'), Esc, Esc]; "clear incomplete state first")]
    fn exit(key_codes: &[KeyCode]) {
        let mut app_mock = MockApplication::default();
        app_mock.expect_on_exit().once().return_const(());
        handle_key_series(key_codes, &mut app_mock);
    }
}
