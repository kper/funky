#![allow(dead_code)]

use std::io;
use std::sync::mpsc;
use std::thread;

use termion::input::TermRead;
use termion::{event::Key};

pub struct Events {
    rx: mpsc::Receiver<Key>,
    input_handle: thread::JoinHandle<()>,
}

impl Events {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let input_handle = {
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
            })
        };

        Events { rx, input_handle }
    }

    pub fn next(&self) -> Result<Key, mpsc::RecvError> {
        self.rx.recv()
    }
}
