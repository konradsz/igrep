use std::path::PathBuf;

pub struct SearchConfig {
    pub pattern: String,
    pub path: PathBuf,
    pub case_insensitive: bool,
    pub case_smart: bool,
}

impl SearchConfig {
    pub fn from(pattern: &str, path: &str) -> Self {
        Self {
            pattern: pattern.into(),
            path: PathBuf::from(path),
            case_insensitive: false,
            case_smart: false,
        }
    }

    pub fn case_insensitive(mut self, case_insensitive: bool) -> Self {
        self.case_insensitive = case_insensitive;
        self
    }

    pub fn case_smart(mut self, case_smart: bool) -> Self {
        self.case_smart = case_smart;
        self
    }
}
