/// Trait for reporting progress during packing operations.
/// Implementors should provide a method to handle progress updates,
/// where progress is reported as a float between 0.0 and 1.0.
pub trait Progress {
    /// Reports the current progress of the operation.
    ///
    /// # Arguments
    /// * `progress` - A float between 0.0 and 1.0 representing the completion percentage
    fn report_progress(&self, progress: f64);
}

// Example implementation using a closure
pub struct ProgressCallback<F>
where
    F: Fn(f64),
{
    callback: F,
}

impl<F> ProgressCallback<F>
where
    F: Fn(f64),
{
    pub fn new(callback: F) -> Self {
        Self { callback }
    }
}

impl<F> Progress for ProgressCallback<F>
where
    F: Fn(f64),
{
    fn report_progress(&self, progress: f64) {
        (self.callback)(progress);
    }
}
