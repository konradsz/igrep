pub mod file_entry;
pub mod grep_match;
pub mod search_config;
mod searcher;
mod sink;

use std::process::ExitStatus;
use std::sync::mpsc;

use crate::editor::EditorCommand;
use crate::ui::result_list::ResultList;
pub use search_config::SearchConfig;
pub use search_config::SortKey;
use searcher::Event;

use self::file_entry::FileEntry;

#[derive(PartialEq, Eq)]
pub enum State {
    Idle,
    Searching,
    OpenFile(bool),
    Error(String),
    Exit,
}

pub struct Ig {
    tx: mpsc::Sender<Event>,
    rx: mpsc::Receiver<Event>,
    state: State,
    editor_command: EditorCommand,
}

impl Ig {
    pub fn new(editor_command: EditorCommand) -> Self {
        let (tx, rx) = mpsc::channel();

        Self {
            tx,
            rx,
            state: State::Idle,
            editor_command,
        }
    }

    fn try_spawn_editor(&self, file_name: &str, line_number: u64) -> anyhow::Result<ExitStatus> {
        let mut editor_process = self.editor_command.spawn(file_name, line_number)?;
        editor_process.wait().map_err(anyhow::Error::from)
    }

    pub fn open_file_if_requested(&mut self, selected_entry: Option<(String, u64)>) {
        if let State::OpenFile(idle) = self.state {
            if let Some((ref file_name, line_number)) = selected_entry {
                match self.try_spawn_editor(file_name, line_number) {
                    Ok(_) => self.state = if idle { State::Idle } else { State::Searching },
                    Err(_) => {
                        self.state = State::Error(format!(
                            "Failed to open editor '{}'. Is it installed?",
                            self.editor_command,
                        ))
                    }
                }
            } else {
                self.state = if idle { State::Idle } else { State::Searching };
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

    pub fn search(&mut self, search_config: SearchConfig, result_list: &mut ResultList) {
        if self.state == State::Idle {
            *result_list = ResultList::default();
            self.state = State::Searching;
            searcher::search(search_config, self.tx.clone());
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
