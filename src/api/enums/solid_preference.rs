/// User per-file preference as to how an individual file should be handled.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u8)]
pub enum SolidPreference {
    // Pack into solid block if can fit into solid block size.
    Default = 0,

    /// This file must be non-SOLIDly packed.
    NoSolid = 1,
}
