use super::*;

/// Trait for items which can provide bytes corresponding to a file.
pub trait CanProvideInputData {
    // Item which provides file data to the user.
    fn input_data_provider(&self) -> &dyn InputDataProvider;
}
