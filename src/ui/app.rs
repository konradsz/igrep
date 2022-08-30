use super::{
    context_viewer::{ContextViewer, ContextViewerState},
    editor::Editor,
    input_handler::{InputHandler, InputState},
    result_list::ResultList,
    scroll_offset_list::{List, ListItem, ListState, ScrollOffset},
    theme::Theme,
};

use crate::{
    file_entry::EntryType,
    ig::{Ig, SearchConfig},
};
use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::path::PathBuf;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame, Terminal,
};

pub struct App {
    ig: Ig,
    result_list: ResultList,
    result_list_state: ListState,
    context_viewer_state: ContextViewerState,
    theme: Box<dyn Theme>,
}

impl App {
    pub fn new(config: SearchConfig, editor: Editor, theme: Box<dyn Theme>) -> Self {
        Self {
            ig: Ig::new(config, editor),
            result_list: ResultList::default(),
            result_list_state: ListState::default(),
            context_viewer_state: ContextViewerState::default(),
            theme,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut input_handler = InputHandler::default();
        self.ig.search(&mut self.result_list);

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

            while self.ig.is_searching() || self.ig.is_idle() {
                terminal.draw(|f| self.draw(f, &input_handler))?;

                if let Some(entry) = self.ig.handle_searcher_event() {
                    self.result_list.add_entry(entry);
                }
                input_handler.handle_input(self)?;

                if let Some((file_name, _)) = self.result_list.get_selected_entry() {
                    if let Some(context_viewer) = self.context_viewer_state.viewer() {
                        context_viewer.highlight_file_if_needed(
                            &PathBuf::from(file_name),
                            self.theme.as_ref(),
                        );
                    }
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
        &mut self,
        frame: &mut Frame<CrosstermBackend<std::io::Stdout>>,
        input_handler: &InputHandler,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
            .split(frame.size());

        let (view_area, bottom_bar_area) = (chunks[0], chunks[1]);

        match &self.context_viewer_state {
            ContextViewerState::None => {
                Self::draw_list(
                    frame,
                    view_area,
                    &self.result_list,
                    &mut self.result_list_state,
                    self.theme.as_ref(),
                );
            }
            ContextViewerState::Vertical(context_viewer) => {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(view_area);

                let (left, right) = (chunks[0], chunks[1]);

                Self::draw_list(
                    frame,
                    left,
                    &self.result_list,
                    &mut self.result_list_state,
                    self.theme.as_ref(),
                );
                Self::draw_context_viewer(
                    frame,
                    right,
                    &self.result_list,
                    context_viewer,
                    self.theme.as_ref(),
                );
            }
            ContextViewerState::Horizontal(context_viewer) => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                    .split(view_area);

                let (top, bottom) = (chunks[0], chunks[1]);

                Self::draw_list(
                    frame,
                    top,
                    &self.result_list,
                    &mut self.result_list_state,
                    self.theme.as_ref(),
                );
                Self::draw_context_viewer(
                    frame,
                    bottom,
                    &self.result_list,
                    context_viewer,
                    self.theme.as_ref(),
                );
            }
        }

        Self::draw_bottom_bar(
            frame,
            bottom_bar_area,
            &self.result_list,
            &self.ig,
            input_handler,
            self.theme.as_ref(),
        );
    }

    fn draw_list(
        frame: &mut Frame<CrosstermBackend<std::io::Stdout>>,
        area: Rect,
        result_list: &ResultList,
        result_list_state: &mut ListState,
        theme: &dyn Theme,
    ) {
        let files_list: Vec<ListItem> = result_list
            .iter()
            .map(|e| match e {
                EntryType::Header(h) => {
                    let h = h.trim_start_matches("./");
                    ListItem::new(Span::styled(h, theme.file_path_color()))
                }
                EntryType::Match(n, t, offsets) => {
                    let line_number = Span::styled(format!(" {}: ", n), theme.line_number_color());

                    let mut spans = vec![line_number];

                    let mut current_position = 0;
                    for offset in offsets {
                        let before_match =
                            Span::styled(&t[current_position..offset.0], theme.list_font_color());
                        let actual_match =
                            Span::styled(&t[offset.0..offset.1], theme.match_color());

                        // set current position to the end of current match
                        current_position = offset.1;

                        spans.push(before_match);
                        spans.push(actual_match);
                    }

                    // push remaining text of a line
                    spans.push(Span::styled(
                        &t[current_position..],
                        theme.list_font_color(),
                    ));

                    ListItem::new(Spans::from(spans))
                }
            })
            .collect();

        let list_widget = List::new(files_list)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .style(theme.background_color())
            .highlight_style(Style::default().bg(theme.highlight_color()))
            .scroll_offset(ScrollOffset::default().top(1).bottom(0));

        result_list_state.select(result_list.get_state().selected());
        frame.render_stateful_widget(list_widget, area, result_list_state);
    }

    fn draw_context_viewer(
        frame: &mut Frame<CrosstermBackend<std::io::Stdout>>,
        area: Rect,
        result_list: &ResultList,
        context_viewer: &ContextViewer,
        theme: &dyn Theme,
    ) {
        let block_widget = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        if let Some((_, line_number)) = result_list.get_selected_entry() {
            let height = area.height as u64;
            let first_line_index = line_number.saturating_sub(height / 2);

            let paragraph_widget = Paragraph::new(context_viewer.get_styled_spans(
                first_line_index as usize,
                height as usize,
                area.width as usize,
                line_number as usize,
                theme,
            ))
            .block(block_widget);

            frame.render_widget(paragraph_widget, area);
        } else {
            frame.render_widget(block_widget, area);
        }
    }

    fn draw_bottom_bar(
        frame: &mut Frame<CrosstermBackend<std::io::Stdout>>,
        area: Rect,
        result_list: &ResultList,
        ig: &Ig,
        input_handler: &InputHandler,
        theme: &dyn Theme,
    ) {
        let current_match_index = result_list.get_current_match_index();

        let (app_status_text, app_status_style) = if ig.is_searching() {
            ("SEARCHING", theme.searching_state_style())
        } else {
            ("FINISHED", theme.finished_state_style())
        };
        let app_status = Span::styled(app_status_text, app_status_style);

        let search_result = Span::raw(if ig.is_searching() {
            "".into()
        } else {
            let total_no_of_matches = result_list.get_total_number_of_matches();
            if total_no_of_matches == 0 {
                " No matches found.".into()
            } else {
                let no_of_files = result_list.get_total_number_of_file_entries();

                let matches_str = if total_no_of_matches == 1 {
                    "match"
                } else {
                    "matches"
                };
                let files_str = if no_of_files == 1 { "file" } else { "files" };

                let filtered_count = result_list.get_filtered_matches_count();
                let filtered_str = if filtered_count != 0 {
                    format!(" ({} filtered out)", filtered_count)
                } else {
                    String::default()
                };

                format!(
                    " Found {} {} in {} {}{}.",
                    total_no_of_matches, matches_str, no_of_files, files_str, filtered_str
                )
            }
        });

        let (current_input_content, current_input_color) = match input_handler.get_state() {
            InputState::Valid => (String::default(), theme.bottom_bar_font_color()),
            InputState::Incomplete(input) => (input.to_owned(), theme.bottom_bar_font_color()),
            InputState::Invalid(input) => (input.to_owned(), theme.invalid_input_color()),
        };
        let current_input = Span::styled(
            current_input_content,
            Style::default()
                .bg(theme.bottom_bar_color())
                .fg(current_input_color),
        );

        let current_no_of_matches = result_list.get_current_number_of_matches();
        let selected_info_text = {
            let width = current_no_of_matches.to_string().len();
            format!(
                " | {: >width$}/{} ",
                current_match_index, current_no_of_matches
            )
        };
        let selected_info_length = selected_info_text.len();
        let selected_info = Span::styled(selected_info_text, theme.bottom_bar_style());

        let hsplit = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length(12),
                    Constraint::Min(1),
                    Constraint::Length(2),
                    Constraint::Length(selected_info_length as u16),
                ]
                .as_ref(),
            )
            .split(area);

        frame.render_widget(
            Paragraph::new(app_status)
                .style(Style::default().bg(app_status_style.bg.expect("Background not set")))
                .alignment(Alignment::Center),
            hsplit[0],
        );

        frame.render_widget(
            Paragraph::new(search_result)
                .style(theme.bottom_bar_style())
                .alignment(Alignment::Left),
            hsplit[1],
        );

        frame.render_widget(
            Paragraph::new(current_input)
                .style(theme.bottom_bar_style())
                .alignment(Alignment::Right),
            hsplit[2],
        );

        frame.render_widget(
            Paragraph::new(selected_info)
                .style(theme.bottom_bar_style())
                .alignment(Alignment::Right),
            hsplit[3],
        );
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
        self.context_viewer_state.toggle_vertical();
    }

    fn on_toggle_context_viewer_horizontal(&mut self) {
        self.context_viewer_state.toggle_horizontal();
    }

    fn on_open_file(&mut self) {
        self.ig.open_file();
    }

    fn on_search(&mut self) {
        self.ig.search(&mut self.result_list);
    }

    fn on_exit(&mut self) {
        self.ig.exit();
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
    fn on_open_file(&mut self);
    fn on_search(&mut self);
    fn on_exit(&mut self);
}
