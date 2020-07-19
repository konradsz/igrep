pub struct Match {
    line_number: u64,
    text: String,
    byte_span: Option<(usize, usize)>,
}

impl Match {
    pub fn new(line_number: u64, text: &str, byte_span: Option<(usize, usize)>) -> Self {
        Match {
            line_number,
            text: text.into(),
            byte_span,
        }
    }
}

pub enum EntryType {
    Header(String),
    Match(u64, String, Option<(usize, usize)>),
}

pub struct FileEntry(pub Vec<EntryType>);

impl FileEntry {
    pub fn new(name: &str, matches: Vec<Match>) -> Self {
        FileEntry(
            std::iter::once(EntryType::Header(name.into()))
                .chain(
                    matches
                        .into_iter()
                        .map(|m| EntryType::Match(m.line_number, m.text, m.byte_span)),
                )
                .collect(),
        )
    }
}
