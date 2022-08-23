use std::{
    borrow::BorrowMut,
    cmp::max,
    io::BufRead,
    path::{Path, PathBuf},
};

use itertools::Itertools;
use syntect::{easy::HighlightFile, highlighting, parsing::SyntaxSet};
use tui::{
    style::{Color, Style},
    text::{Span, Spans},
};

#[derive(Default)]
pub struct ContextViewer {
    file_path: PathBuf,
    file_highlighted: Vec<Vec<(highlighting::Style, String)>>,
}

impl ContextViewer {
    pub fn highlight_file_if_needed(&mut self, file_path: impl AsRef<Path>) {
        if &self.file_path != file_path.as_ref() {
            self.file_path = file_path.as_ref().into();
            self.file_highlighted.clear();

            let ss = SyntaxSet::load_defaults_newlines();
            let ts = highlighting::ThemeSet::load_defaults();

            let mut highlighter =
                HighlightFile::new(file_path, &ss, &ts.themes["base16-ocean.dark"]).unwrap();
            let mut line = String::new();

            while highlighter.reader.read_line(&mut line).unwrap() > 0 {
                let regions: Vec<(highlighting::Style, &str)> = highlighter
                    .highlight_lines
                    .highlight_line(&line, &ss)
                    .unwrap();

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
        total: usize,
        width: usize,
        match_index: usize,
    ) -> Vec<Spans<'_>> {
        let match_offset = match_index - max(first_line_index, 1);

        let mut styled_spans = self
            .file_highlighted
            .iter()
            .skip(first_line_index.saturating_sub(1))
            .take(total)
            .map(|line| {
                line.iter()
                    .map(|(highlight_style, substring)| {
                        let fg = highlight_style.foreground;
                        Span::styled(substring, Style::default().fg(Color::Rgb(fg.r, fg.g, fg.b)))
                    })
                    .collect_vec()
            })
            .map(Spans::from)
            .collect_vec();

        let styled_line = &mut styled_spans[match_offset];
        let line_width = styled_line.width();
        let span_vec = &mut styled_line.0;

        if line_width < width {
            span_vec.push(Span::raw(
                std::iter::repeat(' ')
                    .take(width - line_width)
                    .collect::<String>(),
            ));
        }
        for span in span_vec.into_iter() {
            let current_style = span.style;
            span.borrow_mut().style = current_style.bg(Color::Rgb(23, 30, 102));
        }

        styled_spans
    }
}
