use super::grep_match::GrepMatch;

pub enum EntryType {
    Header(String),
    Match(u64, String, Vec<(usize, usize)>),
}

pub struct FileEntry(Vec<EntryType>);

impl FileEntry {
    pub fn new(name: String, matches: Vec<GrepMatch>) -> Self {
        Self(
            std::iter::once(EntryType::Header(name))
                .chain(matches.into_iter().map(|m| {
                    let mut text = String::new();
                    let mut ofs = m.match_offsets;
                    let mut pos = 0;
                    for c in m.text.chars() {
                        pos += 1;
                        if c != '\t' {
                            text.push(c);
                        } else {
                            text.push_str("  ");
                            for p in &mut ofs {
                                if p.0 >= pos {
                                    p.0 += 1;
                                    p.1 += 1;
                                }
                            }
                        }
                    }
                    EntryType::Match(m.line_number, text, ofs)
                }))
                .collect(),
        )
    }

    pub fn get_matches_count(&self) -> usize {
        self.0
            .iter()
            .filter(|&e| matches!(e, EntryType::Match(_, _, _)))
            .count()
    }

    pub fn get_entries(self) -> Vec<EntryType> {
        self.0
    }
}
