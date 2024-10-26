/// Tuple in FileEntry struct for first 3 presets.
pub mod offset_index_path_tuple_p0_p1_p2;

/// Table of contents header for 'Preset 0', 'Preset 1' and 'Preset 2'.
pub mod preset0_header;
/// Table of contents header for 'Preset 3'
pub mod preset3_header;

/// File entry format for 'Preset 0'
pub mod preset0_fileentry;

/// File entry format for 'Preset 1'
pub mod preset1_fileentry;

/// File entry format for 'Preset 2'
pub mod preset2_fileentry;

/// File entry format for 'Preset 3'
pub mod preset3_fileentry;

/// File entry format for 'Preset 3 w/o Hash'
pub mod preset3_fileentry_nohash;

// Prelude
pub use offset_index_path_tuple_p0_p1_p2::*;
pub use preset0_fileentry::*;
pub use preset0_header::*;
pub use preset1_fileentry::*;
pub use preset2_fileentry::*;
pub use preset3_fileentry::*;
pub use preset3_fileentry_nohash::*;
pub use preset3_header::*;
