use crate::{
    editor::EditorCommand,
    ig::{Ig, SearchConfig},
    ui::{
        bottom_bar, context_viewer::ContextViewer, input_handler::InputHandler,
        keymap_popup::KeymapPopup, result_list::ResultList, search_popup::SearchPopup,
        theme::Theme,
    },
};
use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Frame, Terminal,
};
use std::path::PathBuf;

pub struct App {
    search_config: SearchConfig,
    ig: Ig,
    theme: Box<dyn Theme>,
    result_list: ResultList,
    context_viewer: ContextViewer,
    search_popup: SearchPopup,
    keymap_popup: KeymapPopup,
}

impl App {
    pub fn new(
        search_config: SearchConfig,
        editor_command: EditorCommand,
        context_viewer: ContextViewer,
        theme: Box<dyn Theme>,
    ) -> Self {
        let theme = theme;
        Self {
            search_config,
            ig: Ig::new(editor_command),
            theme,
            context_viewer,
            result_list: ResultList::default(),
            search_popup: SearchPopup::default(),
            keymap_popup: KeymapPopup::default(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut input_handler = InputHandler::default();
        self.ig
            .search(self.search_config.clone(), &mut self.result_list);

        loop {
            let backend = CrosstermBackend::new(std::io::stdout());
            let mut terminal = Terminal::new(backend)?;
            terminal.hide_cursor()?;

            enable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                // NOTE: This is necessary due to upstream `crossterm` requiring that we "enable"
                // mouse handling first, which saves some state that necessary for _disabling_
                // mouse events.
                EnableMouseCapture,
                EnterAlternateScreen,
                DisableMouseCapture
            )?;

            while self.ig.is_searching() || self.ig.last_error().is_some() || self.ig.is_idle() {
                terminal.draw(|f| Self::draw(f, self, &input_handler))?;

                while let Some(entry) = self.ig.handle_searcher_event() {
                    self.result_list.add_entry(entry);
                }

                input_handler.handle_input(self)?;

                if let Some((file_name, _)) = self.result_list.get_selected_entry() {
                    self.context_viewer
                        .update_if_needed(&PathBuf::from(file_name), self.theme.as_ref());
                }
            }

            self.ig
                .open_file_if_requested(self.result_list.get_selected_entry());

            if self.ig.exit_requested() {
                execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                disable_raw_mode()?;
                break;
            }
        }

        Ok(())
    }

    fn draw(
        frame: &mut Frame<CrosstermBackend<std::io::Stdout>>,
        app: &mut App,
        input_handler: &InputHandler,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
            .split(frame.size());

        let (view_area, bottom_bar_area) = (chunks[0], chunks[1]);
        let (list_area, context_viewer_area) = app.context_viewer.split_view(view_area);

        app.result_list.draw(frame, list_area, app.theme.as_ref());

        if let Some(cv_area) = context_viewer_area {
            app.context_viewer
                .draw(frame, cv_area, &app.result_list, app.theme.as_ref());
        }

        bottom_bar::draw(
            frame,
            bottom_bar_area,
            &app.result_list,
            &app.ig,
            input_handler,
            app.theme.as_ref(),
        );

        app.search_popup.draw(frame, app.theme.as_ref());
        app.keymap_popup.draw(frame, app.theme.as_ref());
    }
}

impl Application for App {
    fn is_searching(&self) -> bool {
        self.ig.is_searching()
    }

    fn on_next_match(&mut self) {
        self.result_list.next_match();
    }

    fn on_previous_match(&mut self) {
        self.result_list.previous_match();
    }

    fn on_next_file(&mut self) {
        self.result_list.next_file();
    }

    fn on_previous_file(&mut self) {
        self.result_list.previous_file();
    }

    fn on_top(&mut self) {
        self.result_list.top();
    }

    fn on_bottom(&mut self) {
        self.result_list.bottom();
    }

    fn on_remove_current_entry(&mut self) {
        self.result_list.remove_current_entry();
    }

    fn on_remove_current_file(&mut self) {
        self.result_list.remove_current_file();
    }

    fn on_toggle_context_viewer_vertical(&mut self) {
        self.context_viewer.toggle_vertical();
    }

    fn on_toggle_context_viewer_horizontal(&mut self) {
        self.context_viewer.toggle_horizontal();
    }

    fn on_increase_context_viewer_size(&mut self) {
        self.context_viewer.increase_size();
    }

    fn on_decrease_context_viewer_size(&mut self) {
        self.context_viewer.decrease_size();
    }

    fn on_open_file(&mut self) {
        self.ig.open_file();
    }

    fn on_search(&mut self) {
        let pattern = self.search_popup.get_pattern();
        self.search_config.pattern = pattern;
        self.ig
            .search(self.search_config.clone(), &mut self.result_list);
    }

    fn on_exit(&mut self) {
        self.ig.exit();
    }

    fn on_toggle_popup(&mut self) {
        self.search_popup
            .set_pattern(self.search_config.pattern.clone());
        self.search_popup.toggle();
    }

    fn on_char_inserted(&mut self, c: char) {
        self.search_popup.insert_char(c);
    }

    fn on_char_removed(&mut self) {
        self.search_popup.remove_char();
    }

    fn on_toggle_keymap(&mut self) {
        self.keymap_popup.toggle();
    }

    fn on_keymap_up(&mut self) {
        self.keymap_popup.go_up();
    }

    fn on_keymap_down(&mut self) {
        self.keymap_popup.go_down();
    }

    fn on_keymap_left(&mut self) {
        self.keymap_popup.go_left();
    }

    fn on_keymap_right(&mut self) {
        self.keymap_popup.go_right();
    }
}

#[cfg_attr(test, mockall::automock)]
pub trait Application {
    fn is_searching(&self) -> bool;
    fn on_next_match(&mut self);
    fn on_previous_match(&mut self);
    fn on_next_file(&mut self);
    fn on_previous_file(&mut self);
    fn on_top(&mut self);
    fn on_bottom(&mut self);
    fn on_remove_current_entry(&mut self);
    fn on_remove_current_file(&mut self);
    fn on_toggle_context_viewer_vertical(&mut self);
    fn on_toggle_context_viewer_horizontal(&mut self);
    fn on_increase_context_viewer_size(&mut self);
    fn on_decrease_context_viewer_size(&mut self);
    fn on_open_file(&mut self);
    fn on_search(&mut self);
    fn on_exit(&mut self);
    fn on_toggle_popup(&mut self);
    fn on_char_inserted(&mut self, c: char);
    fn on_char_removed(&mut self);
    fn on_toggle_keymap(&mut self);
    fn on_keymap_up(&mut self);
    fn on_keymap_down(&mut self);
    fn on_keymap_left(&mut self);
    fn on_keymap_right(&mut self);
}
