use std::error::Error;

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, Text},
    Terminal,
};

use crate::entries::{EntryType, FileEntry};

#[derive(Default)]
pub struct ResultList {
    entries: Vec<FileEntry>,
    file_names_indices: Vec<usize>,
    state: tui::widgets::ListState,
}

impl ResultList {
    pub fn add_entry(&mut self, entry: FileEntry) {
        match self.entries.last() {
            Some(e) => {
                let last_header_index = *self.file_names_indices.last().unwrap();
                self.file_names_indices
                    .push(last_header_index + e.matches.len() + 1);
            }
            None => self.file_names_indices.push(0),
        }

        self.entries.push(entry);

        if self.entries.len() == 1 {
            self.state.select(Some(1));
        }
    }

    pub fn render(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<(), Box<dyn Error>> {
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(f.size());

            let header_style = Style::default().fg(Color::Red);

            let files_list =
                self.entries
                    .iter()
                    .map(|item| item.list())
                    .flatten()
                    .map(|e| match e {
                        EntryType::Header(h) => Text::Styled(h.into(), header_style),
                        EntryType::Match(n, t) => Text::raw(format!("{}: {}", n, t)),
                    });

            let list_widget = List::new(files_list)
                .block(Block::default().title("List").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().modifier(Modifier::ITALIC))
                .highlight_symbol(">>");

            f.render_stateful_widget(list_widget, chunks[0], &mut self.state);
        })?;

        Ok(())
    }

    pub fn next(&mut self) {
        if self.entries.is_empty() {
            return;
        }

        let index = match self.state.selected() {
            Some(i) => {
                let next_index = if self.file_names_indices.contains(&(i + 1)) {
                    i + 2
                } else {
                    i + 1
                };

                if next_index
                    >= self.entries.iter().map(|e| e.matches.len()).sum::<usize>()
                        + self.file_names_indices.len()
                {
                    i
                } else {
                    next_index
                }
            }
            None => 1,
        };

        self.state.select(Some(index));
    }

    pub fn previous(&mut self) {
        if self.entries.is_empty() {
            return;
        }

        let index = match self.state.selected() {
            Some(i) => {
                if i == 1 {
                    1
                } else {
                    if self.file_names_indices.contains(&(i - 1)) {
                        i - 2
                    } else {
                        i - 1
                    }
                }
            }
            None => 1,
        };
        self.state.select(Some(index));
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    // todo: return Option<(&str, u64)>
    pub fn get_selected_entry(&self) -> Option<(String, u64)> {
        match self.state.selected() {
            Some(i) => {
                let file_position = self
                    .file_names_indices
                    .iter()
                    .rposition(|&hi| hi < i)
                    .unwrap();
                if let Some(EntryType::Match(n, _)) =
                    self.entries.iter().map(|item| item.list()).flatten().nth(i)
                {
                    return Some((self.entries[file_position].name.clone(), n));
                }
                None
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entries::Match;
    #[test]
    fn test_empty_list() {
        let mut list = ResultList::default();
        assert_eq!(list.state.selected(), None);
        list.next();
        assert_eq!(list.state.selected(), None);
        list.previous();
        assert_eq!(list.state.selected(), None);
    }

    #[test]
    fn test_add_entry() {
        let mut list = ResultList::default();
        list.add_entry(FileEntry::new("entry1", vec![Match::new(0, "e1m1")]));
        assert_eq!(list.entries.len(), 1);
        assert_eq!(list.file_names_indices.len(), 1);
        assert_eq!(list.state.selected(), Some(1));

        list.add_entry(FileEntry::new(
            "entry2",
            vec![Match::new(0, "e1m2"), Match::new(0, "e2m2")],
        ));
        assert_eq!(list.entries.len(), 2);
        assert_eq!(list.file_names_indices.len(), 2);
        assert_eq!(list.state.selected(), Some(1));
    }
}
