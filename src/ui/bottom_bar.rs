use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::Span,
    widgets::Paragraph,
    Frame,
};

use crate::ig::Ig;

use super::{
    input_handler::{InputHandler, InputState},
    result_list::ResultList,
    theme::Theme,
};

pub fn draw(
    frame: &mut Frame<CrosstermBackend<std::io::Stdout>>,
    area: Rect,
    result_list: &ResultList,
    ig: &Ig,
    input_handler: &InputHandler,
    theme: &dyn Theme,
) {
    let (app_status_text, app_status_style) = if ig.is_searching() {
        ("SEARCHING", theme.searching_state_style())
    } else if ig.last_error().is_some() {
        ("ERROR", theme.error_state_style())
    } else {
        ("FINISHED", theme.finished_state_style())
    };
    let app_status = Span::styled(app_status_text, app_status_style);

    let search_result = Span::raw(if ig.is_searching() {
        "".into()
    } else if let Some(err) = ig.last_error() {
        format!(" {err}")
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
                format!(" ({filtered_count} filtered out)")
            } else {
                String::default()
            };

            format!(" Found {total_no_of_matches} {matches_str} in {no_of_files} {files_str}{filtered_str}.")
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
    let current_match_index = result_list.get_current_match_index();
    let selected_info_text = {
        let width = current_no_of_matches.to_string().len();
        format!(" | {current_match_index: >width$}/{current_no_of_matches} ")
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
