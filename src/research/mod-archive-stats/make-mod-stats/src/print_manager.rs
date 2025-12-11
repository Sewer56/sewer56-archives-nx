use std::sync::{Arc, Mutex};

/// Thread-safe print manager for coordinating output across parallel threads
#[derive(Clone)]
pub struct PrintManager {
    inner: Arc<Mutex<()>>,
}

impl PrintManager {
    /// Creates a new PrintManager instance
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(())),
        }
    }

    /// Prints a message in a thread-safe manner
    pub fn println(&self, message: &str) {
        let _guard = self.inner.lock().unwrap();
        println!("{}", message);
    }
}

impl Default for PrintManager {
    fn default() -> Self {
        Self::new()
    }
}
