use core::num::NonZeroU32;

static mut NUM_CORES: Option<NonZeroU32> = None;

/// Retrieves the number of cores that the system has.
pub fn get_num_cores() -> NonZeroU32 {
    // No thread safety needed here (we're running code with no side effects), so we omit lazy_static to save on library space.
    unsafe {
        if NUM_CORES.is_some() {
            return NUM_CORES.unwrap_unchecked();
        }

        #[cfg(feature = "detect_num_cores")]
        {
            NUM_CORES = Some(NonZeroU32::new_unchecked(num_cpus::get_physical() as u32));
        }

        #[cfg(not(feature = "detect_num_cores"))]
        {
            NUM_CORES = Some(NonZeroU32::new_unchecked(1));
        }

        NUM_CORES.unwrap_unchecked()
    }
}
