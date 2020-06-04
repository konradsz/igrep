#[derive(Debug)]
pub struct Match {
    line_number: u64,
    text: String,
}

impl Match {
    pub fn new(line_number: u64, text: &str) -> Self {
        Match {
            line_number,
            text: text.into(),
        }
    }
}

pub enum EntryType<'a> {
    Header(&'a str),
    Match(u64, &'a str),
}

#[derive(Debug)]
pub struct FileEntry {
    pub name: String,
    pub matches: Vec<Match>,
}

impl FileEntry {
    pub fn new(name: &str, matches: Vec<Match>) -> Self {
        FileEntry {
            name: name.into(),
            matches,
        }
    }

    pub fn list(&self) -> impl Iterator<Item = EntryType> {
        let name = std::iter::once(EntryType::Header(self.name.as_str()));
        name.chain(
            self.matches
                .iter()
                .map(|m| EntryType::Match(m.line_number, m.text.as_str())),
        )
    }
}
