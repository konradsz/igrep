#[mockall_double::double]
use super::result_list::ResultList;

use super::result_list::HighlightedFile;

use super::{
    editor::Editor,
    input_handler::{InputHandler, InputState},
    scroll_offset_list::{List, ListItem, ListState, ScrollOffset},
    theme::Theme,
};

#[mockall_double::double]
use crate::ig::Ig;
use crate::{file_entry::EntryType, ig::SearchConfig};
use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::io;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

pub struct App {
    ig: Ig,
    input_handler: InputHandler,
    result_list: ResultList,
    result_list_state: ListState,
    theme: Box<dyn Theme>,
}

impl App {
    pub fn new(config: SearchConfig, editor: Editor, theme: Box<dyn Theme>) -> Self {
        Self {
            ig: Ig::new(config, editor),
            input_handler: InputHandler::default(),
            result_list: ResultList::default(),
            result_list_state: ListState::default(),
            theme,
        }
    }

    fn open_file<B: Backend>(&mut self, term: &mut Terminal<B>) {
        term.clear().unwrap();

        self.ig
            .open_file_if_requested(self.result_list.get_selected_entry());

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
        term.clear().unwrap();
    }

    pub fn run(&mut self) -> Result<()> {
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
                terminal.draw(|f| self.draw(f))?;

                if let Some(entry) = self.ig.handle_searcher_event() {
                    self.result_list.add_entry(entry);
                }
                self.input_handler
                    .handle_input(&mut self.result_list, &mut self.ig)?;

                if self.ig.file_open_requested() {
                    self.open_file(&mut terminal);
                }
            }

            if self.ig.exit_requested() {
                execute!(
                    terminal.backend_mut(),
                    LeaveAlternateScreen,
                    DisableMouseCapture
                )?;
                disable_raw_mode()?;
                break;
            }
        }

        Ok(())
    }

    fn draw(&mut self, f: &mut Frame<CrosstermBackend<std::io::Stdout>>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
            .split(f.size());

        let (top, bottombar_area) = (chunks[0], chunks[1]);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(top);

        let (left, right) = (chunks[0], chunks[1]);

        self.draw_list(f, left);
        self.draw_bottom_bar(f, bottombar_area);
        self.draw_context_viewer(f, right);
    }

    fn draw_list(&mut self, f: &mut Frame<CrosstermBackend<std::io::Stdout>>, area: Rect) {
        let files_list: Vec<ListItem> = self
            .result_list
            .iter()
            .map(|e| match e {
                EntryType::Header(h) => {
                    let h = h.trim_start_matches("./");
                    ListItem::new(Span::styled(h, self.theme.file_path_color()))
                }
                EntryType::Match(n, t, offsets) => {
                    let line_number =
                        Span::styled(format!(" {}: ", n), self.theme.line_number_color());

                    let mut spans = vec![line_number];

                    let mut current_position = 0;
                    for offset in offsets {
                        let before_match = Span::styled(
                            &t[current_position..offset.0],
                            self.theme.list_font_color(),
                        );
                        let actual_match =
                            Span::styled(&t[offset.0..offset.1], self.theme.match_color());

                        // set current position to the end of current match
                        current_position = offset.1;

                        spans.push(before_match);
                        spans.push(actual_match);
                    }

                    // push remaining text of a line
                    spans.push(Span::styled(
                        &t[current_position..],
                        self.theme.list_font_color(),
                    ));

                    ListItem::new(Spans::from(spans))
                }
            })
            .collect();

        let list_widget = List::new(files_list)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(tui::widgets::BorderType::Rounded),
            )
            .style(self.theme.background_color())
            .highlight_style(self.theme.highlight_color())
            .scroll_offset(ScrollOffset::default().top(1).bottom(0));

        self.result_list_state
            .select(self.result_list.get_state().selected());
        f.render_stateful_widget(list_widget, area, &mut self.result_list_state);
    }

    fn make_styled<'a>(&'a self, highlighted: &'a HighlightedFile) -> Vec<Spans<'a>> {
        let mut out = vec![];
        let (_, line_nr) = self.result_list.get_selected_entry().unwrap();

        for (idx, line) in highlighted.iter().enumerate() {
            let spans: Vec<_> = line
                .iter()
                .map(|(hl, s)| {
                    let fg = hl.foreground;
                    let mut style = Style::default().fg(Color::Rgb(fg.r, fg.g, fg.b));

                    if idx + 1 == line_nr as usize {
                        let bg = Color::Rgb(23, 30, 102);
                        style = style.bg(bg);
                    }
                    Span::styled(s, style)
                })
                .collect();

            out.push(Spans::from(spans));
        }

        out
    }

    fn draw_context_viewer(
        &mut self,
        f: &mut Frame<CrosstermBackend<std::io::Stdout>>,
        area: Rect,
    ) {
        let selected_file = &self.result_list.current_file();

        // let height = codeblock.inner(codechunk).height as u64;

        let blocc = Block::default()
            .borders(Borders::ALL)
            .border_type(tui::widgets::BorderType::Rounded);

        if let Some((_, h)) = selected_file {
            let (path, line_nr) = self.result_list.get_selected_entry().unwrap();
            let height = area.height as u64;

            let line_lower = line_nr.saturating_sub(height / 2);

            let line_upper = std::cmp::min(line_lower + height, h.len() as u64);

            let p = Paragraph::new::<Vec<_>>(
                self.make_styled(h)[(line_lower as usize)..(line_upper as usize)]
                    .iter()
                    .map(|l| l.clone())
                    .collect(),
            )
            .wrap(Wrap { trim: false })
            .block(blocc.title(path));

            f.render_widget(p, area);
        } else {
            f.render_widget(blocc, area);
        }
    }

    fn draw_bottom_bar(&mut self, f: &mut Frame<CrosstermBackend<std::io::Stdout>>, area: Rect) {
        let current_match_index = self.result_list.get_current_match_index();

        let (app_status_text, app_status_style) = if self.ig.is_searching() {
            ("SEARCHING", self.theme.searching_state_style())
        } else {
            ("FINISHED", self.theme.finished_state_style())
        };
        let app_status = Span::styled(app_status_text, app_status_style);

        let search_result = Span::raw(if self.ig.is_searching() {
            "".into()
        } else {
            let total_no_of_matches = self.result_list.get_total_number_of_matches();
            if total_no_of_matches == 0 {
                " No matches found.".into()
            } else {
                let no_of_files = self.result_list.get_total_number_of_file_entries();

                let matches_str = if total_no_of_matches == 1 {
                    "match"
                } else {
                    "matches"
                };
                let files_str = if no_of_files == 1 { "file" } else { "files" };

                let filtered_count = self.result_list.get_filtered_matches_count();
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

        let (current_input_content, current_input_color) = match self.input_handler.get_state() {
            InputState::Valid => (String::default(), self.theme.bottom_bar_font_color()),
            InputState::Incomplete(input) => (input.to_owned(), self.theme.bottom_bar_font_color()),
            InputState::Invalid(input) => (input.to_owned(), self.theme.invalid_input_color()),
        };
        let current_input = Span::styled(
            current_input_content,
            Style::default()
                .bg(self.theme.bottom_bar_color())
                .fg(current_input_color),
        );

        let current_no_of_matches = self.result_list.get_current_number_of_matches();
        let selected_info_text = {
            let width = current_no_of_matches.to_string().len();
            format!(
                " | {: >width$}/{} ",
                current_match_index, current_no_of_matches
            )
        };
        let selected_info_length = selected_info_text.len();
        let selected_info = Span::styled(selected_info_text, self.theme.bottom_bar_style());

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

        f.render_widget(
            Paragraph::new(app_status)
                .style(Style::default().bg(app_status_style.bg.expect("Background not set")))
                .alignment(Alignment::Center),
            hsplit[0],
        );

        f.render_widget(
            Paragraph::new(search_result)
                .style(self.theme.bottom_bar_style())
                .alignment(Alignment::Left),
            hsplit[1],
        );

        f.render_widget(
            Paragraph::new(current_input)
                .style(self.theme.bottom_bar_style())
                .alignment(Alignment::Right),
            hsplit[2],
        );

        f.render_widget(
            Paragraph::new(selected_info)
                .style(self.theme.bottom_bar_style())
                .alignment(Alignment::Right),
            hsplit[3],
        );
    }
}
