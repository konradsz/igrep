use std::cmp;

use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders},
    Frame,
};

use crate::ig::file_entry::{EntryType, FileEntry};

use super::{
    scroll_offset_list::{List, ListItem, ListState, ScrollOffset},
    theme::Theme,
};

#[derive(Default)]
pub struct ResultList {
    entries: Vec<EntryType>,
    state: ListState,
    file_entries_count: usize,
    matches_count: usize,
    filtered_matches_count: usize,
}

impl ResultList {
    pub fn add_entry(&mut self, entry: FileEntry) {
        self.file_entries_count += 1;
        self.matches_count += entry.get_matches_count();

        self.entries.append(&mut entry.get_entries());

        if self.state.selected().is_none() {
            self.next_match();
        }
    }

    pub fn iter(&self) -> std::slice::Iter<EntryType> {
        self.entries.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn next_match(&mut self) {
        if self.is_empty() {
            return;
        }

        let index = match self.state.selected() {
            Some(i) => {
                if i == self.entries.len() - 1 {
                    i
                } else {
                    match self.entries[i + 1] {
                        EntryType::Header(_) => i + 2,
                        EntryType::Match(_, _, _) => i + 1,
                    }
                }
            }
            None => 1,
        };

        self.state.select(Some(index));
    }

    pub fn previous_match(&mut self) {
        if self.is_empty() {
            return;
        }

        let index = match self.state.selected() {
            Some(i) => {
                if i == 1 {
                    1
                } else {
                    match self.entries[i - 1] {
                        EntryType::Header(_) => i - 2,
                        EntryType::Match(_, _, _) => i - 1,
                    }
                }
            }
            None => 1,
        };

        self.state.select(Some(index));
    }

    pub fn next_file(&mut self) {
        if self.is_empty() {
            return;
        }

        let index = match self.state.selected() {
            Some(i) => {
                let mut next_index = i;
                loop {
                    if next_index == self.entries.len() - 1 {
                        next_index = i;
                        break;
                    }

                    next_index += 1;
                    match self.entries[next_index] {
                        EntryType::Header(_) => {
                            next_index += 1;
                            break;
                        }
                        EntryType::Match(_, _, _) => continue,
                    }
                }
                next_index
            }
            None => 1,
        };

        self.state.select(Some(index));
    }

    pub fn previous_file(&mut self) {
        if self.is_empty() {
            return;
        }

        let index = match self.state.selected() {
            Some(i) => {
                let mut next_index = i;
                let mut first_header_visited = false;
                loop {
                    if next_index == 1 {
                        break;
                    }

                    next_index -= 1;
                    match self.entries[next_index] {
                        EntryType::Header(_) => {
                            if !first_header_visited {
                                first_header_visited = true;
                                next_index -= 1;
                            } else {
                                next_index += 1;
                                break;
                            }
                        }
                        EntryType::Match(_, _, _) => continue,
                    }
                }
                next_index
            }
            None => 1,
        };

        self.state.select(Some(index));
    }

    pub fn top(&mut self) {
        if self.is_empty() {
            return;
        }

        self.state.select(Some(1));
    }

    pub fn bottom(&mut self) {
        if self.is_empty() {
            return;
        }

        self.state.select(Some(self.entries.len() - 1));
    }

    pub fn remove_current_entry(&mut self) {
        if self.is_empty() {
            return;
        }

        if self.is_last_match_in_file() {
            self.remove_current_file();
        } else {
            self.remove_current_entry_and_select_previous();
        }
    }

    pub fn remove_current_file(&mut self) {
        if self.is_empty() {
            return;
        }

        let selected_index = self.state.selected().expect("Nothing selected");

        let mut current_file_header_index = 0;
        for index in (0..selected_index).rev() {
            if self.is_header(index) {
                current_file_header_index = index;
                break;
            }
        }

        let mut next_file_header_index = self.entries.len();
        for index in selected_index..self.entries.len() {
            if self.is_header(index) {
                next_file_header_index = index;
                break;
            }
        }

        let span = next_file_header_index - current_file_header_index;
        for _ in 0..span {
            self.entries.remove(current_file_header_index);
        }

        self.filtered_matches_count += span - 1;

        if self.entries.is_empty() {
            self.state.select(None);
        } else if selected_index != 1 {
            self.state.select(Some(cmp::max(
                current_file_header_index.saturating_sub(1),
                1,
            )));
        }
    }

    fn is_header(&self, index: usize) -> bool {
        matches!(self.entries[index], EntryType::Header(_))
    }

    fn is_last_match_in_file(&self) -> bool {
        let current_index = self.state.selected().expect("Nothing selected");

        self.is_header(current_index - 1)
            && (current_index == self.entries.len() - 1 || self.is_header(current_index + 1))
    }

    fn remove_current_entry_and_select_previous(&mut self) {
        let selected_index = self.state.selected().expect("Nothing selected");
        self.entries.remove(selected_index);
        self.filtered_matches_count += 1;

        if selected_index >= self.entries.len() || self.is_header(selected_index) {
            self.state.select(Some(selected_index - 1));
        }
    }

    pub fn get_selected_entry(&self) -> Option<(String, u64)> {
        match self.state.selected() {
            Some(i) => {
                let mut line_number: Option<u64> = None;
                for index in (0..=i).rev() {
                    match &self.entries[index] {
                        EntryType::Header(name) => {
                            return Some((
                                name.to_owned(),
                                line_number.expect("Line number not specified"),
                            ));
                        }
                        EntryType::Match(number, _, _) => {
                            if line_number.is_none() {
                                line_number = Some(*number);
                            }
                        }
                    }
                }
                None
            }
            None => None,
        }
    }

    pub fn get_current_match_index(&self) -> usize {
        match self.state.selected() {
            Some(selected) => {
                self.entries
                    .iter()
                    .take(selected)
                    .filter(|&e| matches!(e, EntryType::Match(_, _, _)))
                    .count()
                    + 1
            }
            None => 0,
        }
    }

    pub fn get_current_number_of_matches(&self) -> usize {
        self.entries
            .iter()
            .filter(|&e| matches!(e, EntryType::Match(_, _, _)))
            .count()
    }

    pub fn get_total_number_of_matches(&self) -> usize {
        self.matches_count
    }

    pub fn get_total_number_of_file_entries(&self) -> usize {
        self.file_entries_count
    }

    pub fn get_filtered_matches_count(&self) -> usize {
        self.filtered_matches_count
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect, theme: &dyn Theme) {
        let files_list: Vec<ListItem> = self
            .iter()
            .map(|e| match e {
                EntryType::Header(h) => {
                    let h = h.trim_start_matches("./");
                    ListItem::new(Span::styled(h, theme.file_path_color()))
                }
                EntryType::Match(n, t, offsets) => {
                    let line_number = Span::styled(format!(" {n}: "), theme.line_number_color());

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

                    ListItem::new(Line::from(spans))
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

        let mut state = self.state;
        frame.render_stateful_widget(list_widget, area, &mut state);
        self.state = state;
    }
}

#[cfg(test)]
mod tests {
    use crate::ig::grep_match::GrepMatch;

    use super::*;

    #[test]
    fn test_empty_list() {
        let mut list = ResultList::default();
        assert_eq!(list.state.selected(), None);
        list.next_match();
        assert_eq!(list.state.selected(), None);
        list.previous_match();
        assert_eq!(list.state.selected(), None);
    }

    #[test]
    fn test_add_entry() {
        let mut list = ResultList::default();
        list.add_entry(FileEntry::new(
            "entry1".into(),
            vec![GrepMatch::new(0, "e1m1".into(), vec![])],
        ));
        assert_eq!(list.entries.len(), 2);
        assert_eq!(list.state.selected(), Some(1));

        list.add_entry(FileEntry::new(
            "entry2".into(),
            vec![
                GrepMatch::new(0, "e1m2".into(), vec![]),
                GrepMatch::new(0, "e2m2".into(), vec![]),
            ],
        ));
        assert_eq!(list.entries.len(), 5);
        assert_eq!(list.state.selected(), Some(1));
    }
}
