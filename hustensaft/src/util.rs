use std::io;
use std::sync::mpsc;
use std::thread;

use termion::event::Key;
use termion::input::TermRead;

use tui::widgets::ListState;

use wasm_parser::core::InstructionWrapper;

/// A helper struct to handle terminal input.
pub struct Events {
    rx: mpsc::Receiver<Key>,
}

impl Events {
    pub fn new() -> Events {
        let (tx, rx) = mpsc::channel();
        let tx = tx.clone();
        thread::spawn(move || {
            let stdin = io::stdin();
            for evt in stdin.keys() {
                if let Ok(key) = evt {
                    if let Err(err) = tx.send(key) {
                        eprintln!("{}", err);
                        return;
                    }
                }
            }
        });

        Events { rx }
    }

    pub fn next(&self) -> Result<Key, mpsc::RecvError> {
        self.rx.recv()
    }
}

pub struct StatefulList<'a> {
    pub state: ListState,
    pub items: Vec<&'a InstructionWrapper>,
}

impl<'a> StatefulList<'a> {
    pub fn new() -> StatefulList<'a> {
        StatefulList {
            state: ListState::default(),
            items: Vec::new(),
        }
    }

    pub fn with_items(items: Vec<&InstructionWrapper>) -> StatefulList {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn find_by_id(&mut self, index: usize) {
        let i = match self.state.selected() {
            Some(i) => self
                .items
                .iter()
                .position(|w| w.get_id() == index)
                .unwrap_or(0),
            None => 0,
        };

        self.state.select(Some(i));
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}
