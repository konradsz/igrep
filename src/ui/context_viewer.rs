use std::{
    borrow::BorrowMut,
    cmp::max,
    io::BufRead,
    mem,
    path::{Path, PathBuf},
};

use itertools::Itertools;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};
use syntect::{
    easy::HighlightFile,
    highlighting::{self, ThemeSet},
    parsing::SyntaxSet,
};

use super::theme::Theme;

#[derive(Default, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ContextViewerState {
    #[default]
    None,
    Vertical(ContextViewer),
    Horizontal(ContextViewer),
}

impl ContextViewerState {
    pub fn viewer(&mut self) -> Option<&mut ContextViewer> {
        match self {
            ContextViewerState::None => None,
            ContextViewerState::Vertical(cv) => Some(cv),
            ContextViewerState::Horizontal(cv) => Some(cv),
        }
    }

    pub fn toggle_vertical(&mut self) {
        match self {
            ContextViewerState::None => {
                *self = ContextViewerState::Vertical(ContextViewer::default());
            }
            ContextViewerState::Vertical(_) => {
                *self = ContextViewerState::None;
            }
            ContextViewerState::Horizontal(cv) => {
                *self = ContextViewerState::Vertical(mem::take(cv))
            }
        }
    }

    pub fn toggle_horizontal(&mut self) {
        match self {
            ContextViewerState::None => {
                *self = ContextViewerState::Horizontal(ContextViewer::default());
            }
            ContextViewerState::Vertical(cv) => {
                *self = ContextViewerState::Horizontal(mem::take(cv))
            }
            ContextViewerState::Horizontal(_) => {
                *self = ContextViewerState::None;
            }
        }
    }
}

#[derive(Debug)]
pub struct ContextViewer {
    file_path: PathBuf,
    file_highlighted: Vec<Vec<(highlighting::Style, String)>>,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl Default for ContextViewer {
    fn default() -> Self {
        Self {
            file_path: Default::default(),
            file_highlighted: Default::default(),
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: highlighting::ThemeSet::load_defaults(),
        }
    }
}

impl ContextViewer {
    pub fn highlight_file_if_needed(&mut self, file_path: impl AsRef<Path>, theme: &dyn Theme) {
        if self.file_path != file_path.as_ref() {
            self.file_path = file_path.as_ref().into();
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
    }

    pub fn get_styled_spans(
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

    impl PartialEq for ContextViewer {
        fn eq(&self, other: &Self) -> bool {
            self.file_path == other.file_path && self.file_highlighted == other.file_highlighted
        }
    }

    fn create_cv() -> ContextViewer {
        ContextViewer {
            file_path: "path".into(),
            file_highlighted: vec![vec![(
                highlighting::Style {
                    foreground: highlighting::Color::BLACK,
                    background: highlighting::Color::WHITE,
                    font_style: highlighting::FontStyle::BOLD,
                },
                String::from("line"),
            )]],
            syntax_set: Default::default(),
            theme_set: Default::default(),
        }
    }

    #[test_case(ContextViewerState::None => ContextViewerState::Vertical(ContextViewer::default()))]
    #[test_case(ContextViewerState::Vertical(ContextViewer::default()) => ContextViewerState::None)]
    #[test_case(ContextViewerState::Horizontal(create_cv()) => ContextViewerState::Vertical(create_cv()))]
    fn toggle_vertical(mut context_viewer: ContextViewerState) -> ContextViewerState {
        context_viewer.toggle_vertical();
        context_viewer
    }

    #[test_case(ContextViewerState::None => ContextViewerState::Horizontal(ContextViewer::default()))]
    #[test_case(ContextViewerState::Vertical(create_cv()) => ContextViewerState::Horizontal(create_cv()))]
    #[test_case(ContextViewerState::Horizontal(ContextViewer::default()) => ContextViewerState::None)]
    fn toggle_horizontal(mut context_viewer: ContextViewerState) -> ContextViewerState {
        context_viewer.toggle_horizontal();
        context_viewer
    }
}
