#[derive(Clone, Debug)]
pub struct History {
    prev: Vec<Vec<u8>>,
    next: Vec<Vec<u8>>,
}

impl Default for History {
    fn default() -> Self {
        History {
            prev: Vec::new(),
            next: Vec::new(),
        }
    }
}

impl History {
    pub fn redo(&mut self) -> Option<Vec<u8>> {
        self.next.pop().map(move |elem| {
            self.prev.push(elem.clone());
            elem
        })
    }

    pub fn undo(&mut self) -> Option<Vec<u8>> {
        self.prev.pop().map(move |elem| {
            self.next.push(elem.clone());
            elem
        })
    }

    pub fn update(&mut self, state: Vec<u8>) {
        self.next.clear();
        self.prev.push(state);
    }
}
