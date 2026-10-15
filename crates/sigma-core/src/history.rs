use crate::document::Document;

pub struct History {
    undo_stack: Vec<Document>,
    redo_stack: Vec<Document>,
    max_history: usize,
}

impl Default for History {
    fn default() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 50,
        }
    }
}

impl History {
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history,
        }
    }

    pub fn push_state(&mut self, current_doc: &Document) {
        if self.undo_stack.len() >= self.max_history {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(current_doc.clone());
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, current_doc: &Document) -> Option<Document> {
        let prev = self.undo_stack.pop()?;
        self.redo_stack.push(current_doc.clone());
        Some(prev)
    }

    pub fn redo(&mut self, current_doc: &Document) -> Option<Document> {
        let next = self.redo_stack.pop()?;
        self.undo_stack.push(current_doc.clone());
        Some(next)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}
