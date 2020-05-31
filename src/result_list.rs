use crate::entries::FileEntry;
use tui::widgets::ListState;

pub struct ResultList {
    pub state: ListState,
    pub entries: Vec<FileEntry>,
    header_indices: Vec<usize>,
}

impl ResultList {
    pub fn new() -> ResultList {
        ResultList {
            state: ListState::default(),
            entries: Vec::new(),
            header_indices: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: FileEntry) {
        match self.entries.last() {
            Some(e) => {
                let last_header_index = *self.header_indices.last().unwrap();
                self.header_indices
                    .push(last_header_index + e.matches.len() + 1);
            }
            None => self.header_indices.push(0),
        }

        self.entries.push(entry);

        if self.entries.len() == 1 {
            self.state.select(Some(1));
        }
    }

    pub fn next(&mut self) {
        if self.entries.is_empty() {
            return;
        }

        let index = match self.state.selected() {
            Some(i) => {
                let next_index = if self.header_indices.contains(&(i + 1)) {
                    i + 2
                } else {
                    i + 1
                };

                if next_index
                    >= self.entries.iter().map(|e| e.matches.len()).sum::<usize>()
                        + self.header_indices.len()
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
                    if self.header_indices.contains(&(i - 1)) {
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
}

#[test]
fn test_empty_list() {
    let mut list = ResultList::new();
    assert_eq!(list.state.selected(), None);
    list.next();
    assert_eq!(list.state.selected(), None);
    list.previous();
    assert_eq!(list.state.selected(), None);
}

#[test]
fn test_add_entry() {
    let mut list = ResultList::new();
    list.add_entry(FileEntry::new("entry1", vec![Match::new("e1m1")]));
    assert_eq!(list.entries.len(), 1);
    assert_eq!(list.header_indices.len(), 1);
    assert_eq!(list.state.selected(), Some(1));

    list.add_entry(FileEntry::new(
        "entry2",
        vec![Match::new("e1m2"), Match::new("e2m2")],
    ));
    assert_eq!(list.entries.len(), 2);
    assert_eq!(list.header_indices.len(), 2);
    assert_eq!(list.state.selected(), Some(1));
}
