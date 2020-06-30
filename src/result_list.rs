use crate::entries::{EntryType, FileEntry};

#[derive(Copy, Clone, Default)]
pub struct ListState(Option<usize>);

impl ListState {
    pub fn select(&mut self, index: Option<usize>) {
        self.0 = index;
    }

    pub fn selected(&self) -> Option<usize> {
        self.0
    }
}

#[derive(Default)]
pub struct ResultList {
    entries: Vec<EntryType>,
    state: ListState,
}

impl ResultList {
    pub fn add_entry(&mut self, mut entry: FileEntry) {
        self.entries.append(&mut entry.0);

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
                        EntryType::Match(_, _) => i + 1,
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
                        EntryType::Match(_, _) => i - 1,
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
                        EntryType::Match(_, _) => continue,
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
                        EntryType::Match(_, _) => continue,
                    }
                }
                next_index
            }
            None => 1,
        };

        self.state.select(Some(index));
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

    pub fn get_state(&self) -> ListState {
        self.state
    }

    pub fn get_number_of_matches(&self) -> usize {
        self.entries
            .iter()
            .filter(|&e| match e {
                EntryType::Match(_, _) => true,
                _ => false,
            })
            .count()
    }

    pub fn get_number_of_file_entries(&self) -> usize {
        self.entries
            .iter()
            .filter(|&e| match e {
                EntryType::Header(_) => true,
                _ => false,
            })
            .count()
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
        list.next_match();
        assert_eq!(list.state.selected(), None);
        list.previous_match();
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
