#[derive(Clone, Debug)]
pub struct History {
    init: Vec<u8>,
    prev: Vec<Vec<u8>>,
    next: Vec<Vec<u8>>,
}

impl History {
    pub fn new(init: Vec<u8>) -> Self {
        History {
            init: init,
            prev: Vec::new(),
            next: Vec::new(),
        }
    }
}

impl History {
    pub fn redo(&mut self) -> Option<Vec<u8>> {
        self.next.pop().map(|elem| {
            self.prev.push(elem.clone());
            elem
        })
    }

    pub fn undo(&mut self) -> Option<Vec<u8>> {
        self.prev.pop().map(|elem| { self.next.push(elem); });
        self.prev.pop().map_or(Some(self.init.clone()), |elem| {
            self.prev.push(elem.clone());
            Some(elem)
        })
    }

    pub fn update(&mut self, state: Vec<u8>) {
        self.next.clear();
        self.prev.push(state);
    }
}
