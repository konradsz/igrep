use ratatui::{
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
    frame: &mut Frame,
    area: Rect,
    result_list: &ResultList,
    ig: &Ig,
    input_handler: &InputHandler,
    theme: &dyn Theme,
) {
    let selected_info_text = render_selected_info_text(result_list);

    let hsplit = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(12),
                Constraint::Min(1),
                Constraint::Length(2),
                Constraint::Length(selected_info_text.len() as u16),
            ]
            .as_ref(),
        )
        .split(area);

    draw_app_status(frame, hsplit[0], ig, theme);
    draw_search_result_summary(frame, hsplit[1], ig, result_list, theme);
    draw_current_input(frame, hsplit[2], input_handler, theme);
    draw_selected_info(frame, hsplit[3], selected_info_text, theme);
}

fn draw_app_status(frame: &mut Frame, area: Rect, ig: &Ig, theme: &dyn Theme) {
    let (app_status_text, app_status_style) = if ig.is_searching() {
        ("SEARCHING", theme.searching_state_style())
    } else if ig.last_error().is_some() {
        ("ERROR", theme.error_state_style())
    } else {
        ("FINISHED", theme.finished_state_style())
    };
    let app_status = Span::styled(app_status_text, app_status_style);

    frame.render_widget(
        Paragraph::new(app_status)
            .style(Style::default().bg(app_status_style.bg.expect("Background not set")))
            .alignment(Alignment::Center),
        area,
    );
}

fn draw_search_result_summary(
    frame: &mut Frame,
    area: Rect,
    ig: &Ig,
    result_list: &ResultList,
    theme: &dyn Theme,
) {
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

    frame.render_widget(
        Paragraph::new(search_result)
            .style(theme.bottom_bar_style())
            .alignment(Alignment::Left),
        area,
    );
}

fn draw_current_input(
    frame: &mut Frame,
    area: Rect,
    input_handler: &InputHandler,
    theme: &dyn Theme,
) {
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

    frame.render_widget(
        Paragraph::new(current_input)
            .style(theme.bottom_bar_style())
            .alignment(Alignment::Right),
        area,
    );
}

fn render_selected_info_text(result_list: &ResultList) -> String {
    let current_no_of_matches = result_list.get_current_number_of_matches();
    let current_match_index = result_list.get_current_match_index();
    let width = current_no_of_matches.to_string().len();
    format!(" | {current_match_index: >width$}/{current_no_of_matches} ")
}

fn draw_selected_info(
    frame: &mut Frame,
    area: Rect,
    selected_info_text: String,
    theme: &dyn Theme,
) {
    let selected_info = Span::styled(selected_info_text, theme.bottom_bar_style());

    frame.render_widget(
        Paragraph::new(selected_info)
            .style(theme.bottom_bar_style())
            .alignment(Alignment::Right),
        area,
    );
}
