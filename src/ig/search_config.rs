use std::path::PathBuf;

pub struct SearchConfig {
    pub pattern: String,
    pub path: PathBuf,
    pub case_insensitive: bool,
}

impl SearchConfig {
    pub fn from(pattern: &str, path: &str) -> Self {
        Self {
            pattern: pattern.into(),
            path: PathBuf::from(path),
            case_insensitive: false,
        }
    }

    pub fn case_insensitive(mut self, case_insensitive: bool) -> Self {
        self.case_insensitive = case_insensitive;
        self
    }
}
