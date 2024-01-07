use std::{
    borrow::BorrowMut,
    cmp::max,
    io::BufRead,
    path::{Path, PathBuf},
};

use clap::ValueEnum;
use itertools::Itertools;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};
use syntect::{
    easy::HighlightFile,
    highlighting::{self, ThemeSet},
    parsing::SyntaxSet,
};

use super::{result_list::ResultList, theme::Theme};

#[derive(Default, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum ContextViewerPosition {
    #[default]
    None,
    Vertical,
    Horizontal,
}

#[derive(Debug)]
pub struct ContextViewer {
    highlighted_file_path: PathBuf,
    file_highlighted: Vec<Vec<(highlighting::Style, String)>>,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    position: ContextViewerPosition,
    size: u16,
}

impl ContextViewer {
    const MIN_SIZE: u16 = 20;
    const MAX_SIZE: u16 = 80;
    const SIZE_CHANGE_DELTA: u16 = 5;

    pub fn new(position: ContextViewerPosition) -> Self {
        Self {
            highlighted_file_path: Default::default(),
            file_highlighted: Default::default(),
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: highlighting::ThemeSet::load_defaults(),
            position,
            size: 50,
        }
    }

    pub fn toggle_vertical(&mut self) {
        match self.position {
            ContextViewerPosition::None => self.position = ContextViewerPosition::Vertical,
            ContextViewerPosition::Vertical => self.position = ContextViewerPosition::None,
            ContextViewerPosition::Horizontal => self.position = ContextViewerPosition::Vertical,
        }
    }

    pub fn toggle_horizontal(&mut self) {
        match self.position {
            ContextViewerPosition::None => self.position = ContextViewerPosition::Horizontal,
            ContextViewerPosition::Vertical => self.position = ContextViewerPosition::Horizontal,
            ContextViewerPosition::Horizontal => self.position = ContextViewerPosition::None,
        }
    }

    pub fn increase_size(&mut self) {
        self.size = (self.size + Self::SIZE_CHANGE_DELTA).min(Self::MAX_SIZE);
    }

    pub fn decrease_size(&mut self) {
        self.size = (self.size - Self::SIZE_CHANGE_DELTA).max(Self::MIN_SIZE);
    }

    pub fn update_if_needed(&mut self, file_path: impl AsRef<Path>, theme: &dyn Theme) {
        if self.position == ContextViewerPosition::None
            || self.highlighted_file_path == file_path.as_ref()
        {
            return;
        }

        self.highlighted_file_path = file_path.as_ref().into();
        self.file_highlighted.clear();

        let mut highlighter = HighlightFile::new(
            file_path,
            &self.syntax_set,
            &self.theme_set.themes[theme.context_viewer_theme()],
        )
        .expect("Failed to create line highlighter");
        let mut line = String::new();

        while highlighter
            .reader
            .read_line(&mut line)
            .expect("Not valid UTF-8")
            > 0
        {
            let regions: Vec<(highlighting::Style, &str)> = highlighter
                .highlight_lines
                .highlight_line(&line, &self.syntax_set)
                .expect("Failed to highlight line");

            let span_vec = regions
                .into_iter()
                .map(|(style, substring)| (style, substring.to_string()))
                .collect();

            self.file_highlighted.push(span_vec);
            line.clear(); // read_line appends so we need to clear between lines
        }
    }

    pub fn split_view(&self, view_area: Rect) -> (Rect, Option<Rect>) {
        match self.position {
            ContextViewerPosition::None => (view_area, None),
            ContextViewerPosition::Vertical => {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(100 - self.size),
                        Constraint::Percentage(self.size),
                    ])
                    .split(view_area);

                let (left, right) = (chunks[0], chunks[1]);
                (left, Some(right))
            }
            ContextViewerPosition::Horizontal => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(100 - self.size),
                        Constraint::Percentage(self.size),
                    ])
                    .split(view_area);

                let (top, bottom) = (chunks[0], chunks[1]);
                (top, Some(bottom))
            }
        }
    }

    pub fn draw(
        &self,
        frame: &mut Frame<CrosstermBackend<std::io::Stdout>>,
        area: Rect,
        result_list: &ResultList,
        theme: &dyn Theme,
    ) {
        let block_widget = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        if let Some((_, line_number)) = result_list.get_selected_entry() {
            let height = area.height as u64;
            let first_line_index = line_number.saturating_sub(height / 2);

            let paragraph_widget = Paragraph::new(self.get_styled_spans(
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

    fn get_styled_spans(
        &self,
        first_line_index: usize,
        height: usize,
        width: usize,
        match_index: usize,
        theme: &dyn Theme,
    ) -> Vec<Line<'_>> {
        let mut styled_spans = self
            .file_highlighted
            .iter()
            .skip(first_line_index.saturating_sub(1))
            .take(height)
            .map(|line| {
                line.iter()
                    .map(|(highlight_style, substring)| {
                        let fg = highlight_style.foreground;
                        let substring_without_tab = substring.replace('\t', "    ");
                        Span::styled(
                            substring_without_tab,
                            Style::default().fg(Color::Rgb(fg.r, fg.g, fg.b)),
                        )
                    })
                    .collect_vec()
            })
            .map(Line::from)
            .collect_vec();

        let match_offset = match_index - max(first_line_index, 1);
        let styled_line = &mut styled_spans[match_offset];
        let line_width = styled_line.width();
        let span_vec = &mut styled_line.spans;

        if line_width < width {
            span_vec.push(Span::raw(" ".repeat(width - line_width)));
        }

        for span in span_vec.iter_mut() {
            let current_style = span.style;
            span.borrow_mut().style = current_style.bg(theme.highlight_color());
        }

        styled_spans
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(ContextViewerPosition::None => ContextViewerPosition::Vertical)]
    #[test_case(ContextViewerPosition::Vertical => ContextViewerPosition::None)]
    #[test_case(ContextViewerPosition::Horizontal => ContextViewerPosition::Vertical)]
    fn toggle_vertical(initial_position: ContextViewerPosition) -> ContextViewerPosition {
        let mut context_viewer = ContextViewer::new(initial_position);
        context_viewer.toggle_vertical();
        context_viewer.position
    }

    #[test_case(ContextViewerPosition::None => ContextViewerPosition::Horizontal)]
    #[test_case(ContextViewerPosition::Vertical => ContextViewerPosition::Horizontal)]
    #[test_case(ContextViewerPosition::Horizontal => ContextViewerPosition::None)]
    fn toggle_horizontal(initial_position: ContextViewerPosition) -> ContextViewerPosition {
        let mut context_viewer = ContextViewer::new(initial_position);
        context_viewer.toggle_horizontal();
        context_viewer.position
    }

    #[test]
    fn increase_size() {
        let mut context_viewer = ContextViewer::new(ContextViewerPosition::None);
        let default_size = context_viewer.size;
        context_viewer.increase_size();
        assert_eq!(
            context_viewer.size,
            default_size + ContextViewer::SIZE_CHANGE_DELTA
        );

        context_viewer.size = ContextViewer::MAX_SIZE;
        context_viewer.increase_size();
        assert_eq!(context_viewer.size, ContextViewer::MAX_SIZE);
    }

    #[test]
    fn decrease_size() {
        let mut context_viewer = ContextViewer::new(ContextViewerPosition::None);
        let default_size = context_viewer.size;
        context_viewer.decrease_size();
        assert_eq!(
            context_viewer.size,
            default_size - ContextViewer::SIZE_CHANGE_DELTA
        );

        context_viewer.size = ContextViewer::MIN_SIZE;
        context_viewer.decrease_size();
        assert_eq!(context_viewer.size, ContextViewer::MIN_SIZE);
    }
}
