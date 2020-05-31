pub struct Match(String);

impl Match {
    pub fn new(text: &str) -> Self {
        Match(text.into())
    }
}

pub enum Type<'a> {
    Header(&'a str),
    Match(&'a str),
}

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

    pub fn list(&self) -> impl Iterator<Item = Type> {
        let name = std::iter::once(Type::Header(self.name.as_str()));
        name.chain(self.matches.iter().map(|m| Type::Match(m.0.as_str())))
    }
}
