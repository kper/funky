use crate::PAGE_SIZE;

#[derive(Debug, Clone, Copy)]
pub struct Page(usize);

impl Page {
    pub fn new(n: usize) -> Self {
        Self(n)
    }

    pub fn from_count(c: usize) -> Self {
        Self(c / PAGE_SIZE)
    }

    pub fn pages(&self) -> usize {
        self.0
    }

    pub fn elements(&self) -> usize {
        self.pages() * PAGE_SIZE
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl std::ops::Add for Page {
    type Output = Page;

    fn add(self, other: Page) -> Page {
        Page(self.0 + other.0)
    }
}
