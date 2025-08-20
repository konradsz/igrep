use anyhow::Result;
use ignore::{
    overrides::{Override, OverrideBuilder},
    types::{Types, TypesBuilder},
};
use std::path::PathBuf;

use crate::args::SortKeyArg;

#[derive(Clone, Copy)]
pub enum SortKey {
    Path,
    PathReversed,
    Modified,
    ModifiedReversed,
    Created,
    CreatedReversed,
    Accessed,
    AccessedReversed,
}

#[derive(Clone)]
pub struct SearchConfig {
    pub pattern: String,
    pub paths: Vec<PathBuf>,
    pub case_insensitive: bool,
    pub case_smart: bool,
    pub overrides: Override,
    pub types: Types,
    pub search_hidden: bool,
    pub follow_links: bool,
    pub word_regexp: bool,
    pub sort_by: Option<SortKey>,
    pub fixed_strings: bool,
}

impl SearchConfig {
    pub fn from(pattern: String, paths: Vec<PathBuf>) -> Result<Self> {
        let mut builder = TypesBuilder::new();
        builder.add_defaults();
        let types = builder.build()?;

        Ok(Self {
            pattern,
            paths,
            case_insensitive: false,
            case_smart: false,
            overrides: Override::empty(),
            types,
            search_hidden: false,
            follow_links: false,
            word_regexp: false,
            fixed_strings: false,
            sort_by: None,
        })
    }

    pub fn case_insensitive(mut self, case_insensitive: bool) -> Self {
        self.case_insensitive = case_insensitive;
        self
    }

    pub fn case_smart(mut self, case_smart: bool) -> Self {
        self.case_smart = case_smart;
        self
    }

    pub fn globs(mut self, globs: Vec<String>) -> Result<Self> {
        let mut builder = OverrideBuilder::new(std::env::current_dir()?);
        for glob in globs {
            builder.add(&glob)?;
        }
        self.overrides = builder.build()?;
        Ok(self)
    }

    pub fn file_types(
        mut self,
        file_types: Vec<String>,
        file_types_not: Vec<String>,
    ) -> Result<Self> {
        let mut builder = TypesBuilder::new();
        builder.add_defaults();
        for file_type in file_types {
            builder.select(&file_type);
        }
        for file_type in file_types_not {
            builder.negate(&file_type);
        }
        self.types = builder.build()?;
        Ok(self)
    }

    pub fn sort_by(
        mut self,
        sort_by: Option<SortKeyArg>,
        sort_by_reversed: Option<SortKeyArg>,
    ) -> Result<Self> {
        if let Some(arg) = sort_by {
            match arg {
                SortKeyArg::Path => self.sort_by = Some(SortKey::Path),
                SortKeyArg::Modified => self.sort_by = Some(SortKey::Modified),
                SortKeyArg::Created => self.sort_by = Some(SortKey::Created),
                SortKeyArg::Accessed => self.sort_by = Some(SortKey::Accessed),
            }
        };
        if let Some(arg) = sort_by_reversed {
            match arg {
                SortKeyArg::Path => self.sort_by = Some(SortKey::PathReversed),
                SortKeyArg::Modified => self.sort_by = Some(SortKey::ModifiedReversed),
                SortKeyArg::Created => self.sort_by = Some(SortKey::CreatedReversed),
                SortKeyArg::Accessed => self.sort_by = Some(SortKey::AccessedReversed),
            }
        };
        Ok(self)
    }

    pub fn search_hidden(mut self, search_hidden: bool) -> Self {
        self.search_hidden = search_hidden;
        self
    }

    pub fn follow_links(mut self, follow_links: bool) -> Self {
        self.follow_links = follow_links;
        self
    }

    pub fn word_regexp(mut self, word_regexp: bool) -> Self {
        self.word_regexp = word_regexp;
        self
    }

    pub fn fixed_strings(mut self, fixed_strings: bool) -> Self {
        self.fixed_strings = fixed_strings;
        self
    }
}
