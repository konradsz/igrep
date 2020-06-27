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
    entries: Vec<EntryType>,
    state: tui::widgets::ListState,
}

impl ResultList {
    pub fn add_entry(&mut self, mut entry: FileEntry) {
        self.entries.append(&mut entry.0);

        if self.state.selected().is_none() {
            self.next();
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

            let files_list = self.entries.iter().map(|e| match e {
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
                if i == self.entries.len() - 1 {
                    i
                } else {
                    match self.entries[i + 1] {
                        EntryType::Header(_) => i + 2,
                        EntryType::Match(_, _) => i + 1,
                    }
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
                    match self.entries[i - 1] {
                        EntryType::Header(_) => i - 2,
                        EntryType::Match(_, _) => i - 1,
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

    pub fn get_selected_entry(&self) -> Option<(&str, u64)> {
        match self.state.selected() {
            Some(i) => {
                let mut line_number: Option<u64> = None;
                for index in (0..=i).rev() {
                    match &self.entries[index] {
                        EntryType::Header(name) => {
                            return Some((name.as_str(), line_number.unwrap()));
                        }
                        EntryType::Match(number, _) => {
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
        assert_eq!(list.entries.len(), 2);
        assert_eq!(list.state.selected(), Some(1));

        list.add_entry(FileEntry::new(
            "entry2",
            vec![Match::new(0, "e1m2"), Match::new(0, "e2m2")],
        ));
        assert_eq!(list.entries.len(), 5);
        assert_eq!(list.state.selected(), Some(1));
    }
}
