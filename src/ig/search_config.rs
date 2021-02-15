use ignore::types::{Types, TypesBuilder};
use std::path::PathBuf;

pub struct SearchConfig {
    pub pattern: String,
    pub path: PathBuf,
    pub case_insensitive: bool,
    pub case_smart: bool,
    pub types: Types,
}

impl SearchConfig {
    pub fn from(pattern: &str, path: &str) -> Self {
        let mut builder = TypesBuilder::new();
        builder.add_defaults();
        let types = builder.build().unwrap();

        Self {
            pattern: pattern.into(),
            path: PathBuf::from(path),
            case_insensitive: false,
            case_smart: false,
            types,
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

    pub fn file_types(mut self, file_types: Vec<&str>, file_types_not: Vec<&str>) -> Self {
        let mut builder = TypesBuilder::new();
        builder.add_defaults();
        for file_type in file_types {
            builder.select(file_type);
        }
        for file_type in file_types_not {
            builder.negate(file_type);
        }
        self.types = builder.build().unwrap();
        self
    }
}
