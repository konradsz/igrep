pub mod event;

use tui::widgets::ListState;

pub struct Match(String);

impl Match {
    pub fn new(text: &str) -> Self {
        Match(text.into())
    }
}

pub struct FileEntry {
    name: String,
    matches: Vec<Match>,
}

impl FileEntry {
    pub fn new(name: &str, matches: Vec<Match>) -> Self {
        FileEntry {
            name: name.into(),
            matches,
        }
    }

    pub fn list(&self) -> impl Iterator<Item = &str> {
        let name = std::iter::once(self.name.as_str());
        name.chain(self.matches.iter().map(|m| m.0.as_str()))
    }
}

pub struct StatefulList {
    pub state: ListState,
    pub entries: Vec<FileEntry>,
    headers: Vec<usize>,
}

impl StatefulList {
    pub fn new() -> StatefulList {
        StatefulList {
            state: ListState::default(),
            entries: Vec::new(),
            headers: Vec::new(), // header_indices
        }
    }

    pub fn add_entry(&mut self, entry: FileEntry) {
        match self.entries.last() {
            Some(e) => {
                let last_header_index = *self.headers.last().unwrap();
                self.headers.push(last_header_index + e.matches.len() + 1);
            }
            None => self.headers.push(0),
        }
        self.entries.push(entry);
    }

    pub fn next(&mut self) {
        if self.entries.is_empty() {
            self.state.select(None);
            return;
        }

        let index = match self.state.selected() {
            Some(i) => {
                let next_index = if self.headers.contains(&(i + 1)) {
                    i + 2
                } else {
                    i + 1
                };

                if next_index
                    >= self.entries.iter().map(|e| e.matches.len()).sum::<usize>()
                        + self.headers.len()
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
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.entries.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}

#[test]
fn test_empty_list() {
    let mut list = StatefulList::new();
    assert_eq!(list.state.selected(), None);
    list.next();
    assert_eq!(list.state.selected(), None);
}
