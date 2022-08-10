use crate::file_entry::{EntryType, FileEntry};
use std::cmp;
use syntect::highlighting as highl;
use syntect::parsing::SyntaxSet;
use syntect::easy::HighlightFile;
use std::io::BufRead;

pub type HighlightedFile = Vec<Vec<(highl::Style, String)>>;


fn highlight_file(file_path: &str) -> HighlightedFile {
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = highl::ThemeSet::load_defaults();

    let mut highlighter = HighlightFile::new(file_path, &ss, &ts.themes["base16-ocean.dark"]).unwrap();
    let mut line = String::new();

    let mut out: HighlightedFile = Vec::new();
    while highlighter.reader.read_line(&mut line).unwrap() > 0 {
        {
            let regions: Vec<(highl::Style, &str)> = highlighter.highlight_lines.highlight_line(&line, &ss).unwrap();
            // out.push_str(as_24_bit_terminal_escaped(&regions[..], false).as_str());
            let span_vec = regions.into_iter().map(|(hl, s)|{
                (hl, s.to_string())
            });
            out.push(span_vec.collect());

            // print!("{}", as_24_bit_terminal_escaped(&regions[..], true));
        } // until NLL this scope is needed so we can clear the buffer after
        line.clear(); // read_line appends so we need to clear between lines
    }

    out

}



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
    current_file: Option<(String, HighlightedFile)>,
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

    pub fn clear(&mut self) {
        self.entries.clear();
        self.file_entries_count = 0;
        self.matches_count = 0;
        self.filtered_matches_count = 0;
        self.state.select(None);
        self.make_highlighted();
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
        self.make_highlighted();
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
        self.make_highlighted();
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
        self.make_highlighted();

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
        self.make_highlighted();

    }

    pub fn top(&mut self) {
        if self.is_empty() {
            return;
        }

        self.state.select(Some(1));
        self.make_highlighted();
    }

    pub fn bottom(&mut self) {
        if self.is_empty() {
            return;
        }

        self.state.select(Some(self.entries.len() - 1));
        self.make_highlighted();
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
        self.make_highlighted();
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
        self.make_highlighted();
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

    pub fn make_highlighted(&mut self) {
        if let Some((file_path, _)) = self.get_selected_entry(){
            if self.current_file.is_none() 
                || self.current_file.as_ref().unwrap().0 != file_path {
                self.current_file = Some((file_path.clone(), highlight_file(file_path.as_str())));
            }
        } else {
            self.current_file = None;
        }

    }

    pub fn get_state(&self) -> ListState {
        self.state
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

    pub fn current_file(&self) -> &Option<(String, HighlightedFile)> {
        &self.current_file
    }
}

#[cfg(test)]
mockall::mock! {
    pub ResultList {
        pub fn current_file(&self) -> &Option<(String, HighlightedFile)>;
        pub fn add_entry(&mut self, entry: FileEntry);
        pub fn iter<'a>(&self) -> std::slice::Iter<'a, EntryType>;
        pub fn clear(&mut self);
        pub fn is_empty(&self) -> bool;
        pub fn next_match(&mut self);
        pub fn previous_match(&mut self);
        pub fn next_file(&mut self);
        pub fn previous_file(&mut self);
        pub fn top(&mut self);
        pub fn bottom(&mut self);
        pub fn remove_current_entry(&mut self);
        pub fn remove_current_file(&mut self);
        pub fn get_selected_entry(&self) -> Option<(String, u64)>;
        pub fn get_state(&self) -> ListState;
        pub fn get_current_match_index(&self) -> usize;
        pub fn get_current_number_of_matches(&self) -> usize;
        pub fn get_total_number_of_matches(&self) -> usize;
        pub fn get_total_number_of_file_entries(&self) -> usize;
        pub fn get_filtered_matches_count(&self) -> usize;
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grep_match::GrepMatch;

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
