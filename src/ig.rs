use std::sync::mpsc;

use crate::result_list::ResultList;
use crate::scroll_offset_list::ListState;
use crate::searcher::{Event, SearchConfig, Searcher};

use crate::app::AppState;

pub struct Ig {
    rx: mpsc::Receiver<Event>,
    pub state: AppState,
    searcher: Searcher,
    pub result_list: ResultList,
    pub result_list_state: ListState,
}

impl Ig {
    pub fn new(config: SearchConfig) -> Self {
        let (tx, rx) = mpsc::channel();

        Self {
            rx,
            state: AppState::Idle,
            searcher: Searcher::new(config, tx),
            result_list: ResultList::default(),
            result_list_state: ListState::default(),
        }
    }

    pub fn handle_searcher_event(&mut self) {
        if let Ok(event) = self.rx.try_recv() {
            match event {
                Event::NewEntry(e) => self.result_list.add_entry(e),
                Event::SearchingFinished => self.state = AppState::Idle,
            }
        }
    }

    pub fn search(&mut self) {
        self.state = AppState::Searching;
        self.searcher.search();
    }

    pub fn open_file(&mut self) {
        self.state = AppState::OpenFile(self.state == AppState::Idle);
    }

    pub fn exit(&mut self) {
        self.state = AppState::Exit;
    }

    pub fn is_idle(&self) -> bool {
        self.state == AppState::Idle
    }
}
