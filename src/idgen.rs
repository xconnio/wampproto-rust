use std::sync::Mutex;

const MAX_ID: i64 = 1 << 53;

#[derive(Debug)]
pub struct SessionScopeIDGenerator {
    id: Mutex<i64>,
}

impl SessionScopeIDGenerator {
    pub fn new() -> Self {
        Self { id: Mutex::new(0) }
    }

    pub fn next_id(&self) -> i64 {
        let mut id = self.id.lock().unwrap();

        if *id == MAX_ID {
            *id = 0;
        }

        *id += 1;
        *id
    }
}

impl Default for SessionScopeIDGenerator {
    fn default() -> Self {
        Self::new()
    }
}
