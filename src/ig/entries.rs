pub struct Match {
    line_number: u64,
    text: String,
    match_offsets: Vec<(usize, usize)>,
}

impl Match {
    pub fn new(line_number: u64, text: &str, match_offsets: Vec<(usize, usize)>) -> Self {
        Match {
            line_number,
            text: text.into(),
            match_offsets,
        }
    }
}

pub enum EntryType {
    Header(String),
    Match(u64, String, Vec<(usize, usize)>),
}

pub struct FileEntry(pub Vec<EntryType>);

impl FileEntry {
    pub fn new(name: &str, matches: Vec<Match>) -> Self {
        FileEntry(
            std::iter::once(EntryType::Header(name.into()))
                .chain(
                    matches
                        .into_iter()
                        .map(|m| EntryType::Match(m.line_number, m.text, m.match_offsets)),
                )
                .collect(),
        )
    }
}
