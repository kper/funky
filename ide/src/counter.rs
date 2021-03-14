#![allow(dead_code)]

use anyhow::{Context, Result};
use log::debug;

#[derive(Debug, Default)]
pub struct Counter {
    counter: usize,
}

impl Counter {
    pub fn peek(&self) -> usize {
        self.counter
    }

    pub fn get(&mut self) -> usize {
        let counter = self.counter.clone();
        self.counter += 1;
        counter
    }

    pub fn peek_next(&self) -> usize {
        self.counter + 1
    }
}

#[derive(Debug, Default)]
pub struct StackedCounter {
    counter: Vec<Counter>,
    current: usize,
}

impl StackedCounter {
    pub fn peek(&self) -> Result<usize> {
        Ok(self
            .counter
            .get(self.current - 1)
            .context("No function left")?
            .peek())
    }

    pub fn get(&mut self) -> Result<usize> {
        Ok(self
            .counter
            .get_mut(self.current - 1)
            .context("No function left")?
            .get())
    }

    pub fn peek_next(&self) -> Result<usize> {
        Ok(self
            .counter
            .get(self.current - 1)
            .context("No function left")?
            .peek_next())
    }

    pub fn push(&mut self) {
        debug!("Pushing new function to counter");
        self.counter.push(Counter::default());
        self.current += 1;
    }

    pub fn pop(&mut self) {
        debug!("Popping function from counter");
        self.current -= 1;
        self.counter.pop();
    }
}
