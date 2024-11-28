pub trait HasDictIndex {
    /// Index of the dictionary for dictionary compression, if dictionary
    /// compression is being used.
    fn dict_index(&self) -> u32;
}
