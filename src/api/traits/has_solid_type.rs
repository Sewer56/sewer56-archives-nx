use crate::api::enums::solid_preference::SolidPreference;

/// Used for items which can specify a preference on whether they'd prefer to be SOLIDly packed or not.
pub trait HasSolidType {
    /// Preference in terms of whether this item should be SOLID or not.
    fn solid_type(&self) -> SolidPreference;
}
