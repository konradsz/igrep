mod search_config;
mod searcher;
mod sink;

pub use search_config::SearchConfig;

use std::{process::Command, sync::mpsc};

use crate::{
    file_entry::FileEntry,
    ui::{editor::Editor, result_list::ResultList},
};
use searcher::{Event, Searcher};

#[derive(PartialEq)]
pub enum State {
    Idle,
    Searching,
    OpenFile(bool),
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

    pub fn open_file_if_requested(&mut self, selected_entry: Option<(&str, u64)>) {
        if let State::OpenFile(idle) = self.state {
            if let Some((file_name, line_number)) = selected_entry {
                let mut child_process = Command::new(self.editor.to_string())
                    .arg(format!("+{}", line_number))
                    .arg(file_name)
                    .spawn()
                    .expect(&format!(
                        "Error: Failed to run editor with a command: \"{} +{} {}\".",
                        self.editor.to_string(),
                        line_number,
                        file_name
                    ));
                child_process.wait().expect("Error: Editor failed to exit.");
            }

            self.state = if idle { State::Idle } else { State::Searching };
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
            result_list.clear();
            self.state = State::Searching;
            self.searcher.search();
        }
    }

    pub fn open_file(&mut self) {
        self.state = State::OpenFile(self.state == State::Idle);
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

    pub fn exit_requested(&self) -> bool {
        self.state == State::Exit
    }
}
