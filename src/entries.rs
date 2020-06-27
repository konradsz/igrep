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

pub enum EntryType {
    Header(String),
    Match(u64, String),
}

pub struct FileEntry(pub Vec<EntryType>);

impl FileEntry {
    pub fn new(name: &str, matches: Vec<Match>) -> Self {
        FileEntry(
            std::iter::once(EntryType::Header(name.into()))
                .chain(
                    matches
                        .into_iter()
                        .map(|m| EntryType::Match(m.line_number, m.text)),
                )
                .collect(),
        )
    }
}
