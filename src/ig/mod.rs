mod search_config;
mod searcher;
mod sink;

use crate::{
    file_entry::FileEntry,
    ui::{editor::Editor, result_list::ResultList},
};
pub use search_config::SearchConfig;
use searcher::{Event, Searcher};
use std::io;
use std::process::ExitStatus;
use std::sync::mpsc;

#[derive(PartialEq, Eq)]
pub enum State {
    Idle,
    Searching,
    OpenFile(bool),
    Error(String),
    Exit,
}

pub struct Ig {
    rx: mpsc::Receiver<Event>,
    state: State,
    searcher: Searcher,
    editor: Editor,
}

impl Ig {
    pub fn new(config: SearchConfig, editor: Editor) -> Self {
        let (tx, rx) = mpsc::channel();

        Self {
            rx,
            state: State::Idle,
            searcher: Searcher::new(config, tx),
            editor,
        }
    }

    fn try_spawn_editor(&self, file_name: &str, line_number: u64) -> io::Result<ExitStatus> {
        let mut editor_process = self.editor.spawn(&file_name, line_number)?;
        editor_process.wait()
    }

    pub fn open_file_if_requested(&mut self, selected_entry: Option<(String, u64)>) {
        if let State::OpenFile(idle) = self.state {
            if let Some((ref file_name, line_number)) = selected_entry {
                match self.try_spawn_editor(file_name, line_number) {
                    Ok(_) => self.state = if idle { State::Idle } else { State::Searching },
                    Err(_) => {
                        self.state = State::Error(format!(
                            "Failed to open editor '{}'. Is it installed?",
                            self.editor,
                        ))
                    }
                }
            }
        }
    }

    pub fn handle_searcher_event(&mut self) -> Option<FileEntry> {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                Event::NewEntry(e) => return Some(e),
                Event::SearchingFinished => self.state = State::Idle,
                Event::Error => self.state = State::Exit,
            }
        }

        None
    }

    pub fn search(&mut self, result_list: &mut ResultList) {
        if self.state == State::Idle {
            *result_list = ResultList::default();
            self.state = State::Searching;
            self.searcher.search();
        }
    }

    pub fn open_file(&mut self) {
        self.state = State::OpenFile(self.state == State::Idle);
    }

    pub fn error(&mut self, err: String) {
        self.state = State::Error(err);
    }

    pub fn exit(&mut self) {
        self.state = State::Exit;
    }

    pub fn is_idle(&self) -> bool {
        self.state == State::Idle
    }

    pub fn is_searching(&self) -> bool {
        self.state == State::Searching
    }

    pub fn last_error(&self) -> Option<&str> {
        if let State::Error(err) = &self.state {
            Some(err)
        } else {
            None
        }
    }

    pub fn exit_requested(&self) -> bool {
        self.state == State::Exit
    }
}
