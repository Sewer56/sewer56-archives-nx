use crate::utilities::system_info::get_num_cores;

use super::super::NxCompressionError;
use core::{
    ffi::{c_uint, c_void},
    mem::*,
    ptr::*,
};
use zstd_sys::*;

/// Checks if there are enough samples to train a dictionary.
pub fn has_enough_samples_for_dictionary(num_samples: usize) -> bool {
    num_samples >= 7
}

/// Trains a dictionary from sample data for use with ZStandard compression.
///
/// # Parameters
///
/// * `samples`: Slice of sample data buffers to train the dictionary on.
/// * `dict_size`: Maximum size of the resulting dictionary in bytes.
/// * `compression_level`: Level to optimize the dictionary for.
///
/// # Returns
///
/// The trained dictionary data on success.
/// May return [`ZSTD_error_srcSize_wrong`] if there are not enough samples.
pub fn train_dictionary(
    samples: &[&[u8]],
    dict_size: usize,
    compression_level: i32,
) -> Result<Vec<u8>, NxCompressionError> {
    // Calculate total samples size and create buffers
    let total_size: usize = samples.iter().map(|s| s.len()).sum();
    let mut samples_buffer = Vec::with_capacity(total_size);
    let mut sample_sizes = Vec::with_capacity(samples.len());

    unsafe {
        // Fast copy of samples (without bounds checks on every vec append)
        let mut write_ptr = samples_buffer.as_mut_ptr();
        let mut size_ptr = sample_sizes.as_mut_ptr();

        for sample in samples {
            copy_nonoverlapping(sample.as_ptr(), write_ptr, sample.len());
            write(size_ptr, sample.len());

            write_ptr = write_ptr.add(sample.len());
            size_ptr = size_ptr.add(1);
        }

        samples_buffer.set_len(total_size);
        sample_sizes.set_len(samples.len());
    }

    // Allocate dictionary buffer
    let mut dict_buffer = Vec::with_capacity(dict_size);

    unsafe {
        // Set up fastCover parameters with defaults
        let mut cover_params = zeroed::<ZDICT_fastCover_params_t>();
        cover_params.zParams.compressionLevel = compression_level;

        // These params are copied from ZDICT_trainFromBuffer defaults
        cover_params.d = 8;
        cover_params.k = 2048;
        cover_params.steps = 4;
        cover_params.nbThreads = get_num_cores().get() as c_uint;

        // Optimize the dictionary using fastCover
        let optimize_result = ZDICT_optimizeTrainFromBuffer_fastCover(
            dict_buffer.as_mut_ptr() as *mut c_void,
            dict_buffer.capacity(),
            samples_buffer.as_ptr() as *const c_void,
            sample_sizes.as_ptr(),
            samples.len() as c_uint,
            &mut cover_params,
        );

        if ZSTD_isError(optimize_result) != 0 {
            return Err(NxCompressionError::ZStandard(ZSTD_getErrorCode(
                optimize_result,
            )));
        }

        dict_buffer.set_len(optimize_result);
        Ok(dict_buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zstd_sys::ZSTD_ErrorCode::ZSTD_error_srcSize_wrong;

    #[test]
    fn can_train_dictionary() {
        // Create sample data
        let samples: [&[u8]; 7] = [
            b"The energy core has reached critical levels. Immediate evacuation required.",
            b"Warning: Multiple hostile entities detected in the lower maintenance sector.",
            b"Access denied: Security clearance level 4 or higher required for this terminal.",
            b"System log: Quantum stabilizer offline. Containment fields at 67% capacity.",
            b"Coordinates locked. Hyperspace jump sequence initialized. Stand by.",
            b"Incoming transmission from deep space relay station Alpha-Nine.",
            b"Neural interface calibration complete. Pilot synchronization at optimal levels.",
        ];
        assert!(has_enough_samples_for_dictionary(samples.len()));

        // Try to train with various dictionary sizes
        let dict_sizes = [256, 512, 1024, 2048, 4096];
        for dict_size in dict_sizes {
            let dict_data = train_dictionary(&samples, dict_size, 16).unwrap();
            assert!(
                dict_data.len() <= dict_size,
                "Dictionary exceeds requested size"
            );
            assert!(!dict_data.is_empty(), "Dictionary should not be empty");
        }
    }

    #[test]
    fn errors_on_empty_samples() {
        let samples: [&[u8]; 0] = [];
        assert!(!has_enough_samples_for_dictionary(samples.len()));
        let err = train_dictionary(&samples, 1024, 3).unwrap_err();
        assert_eq!(err, NxCompressionError::ZStandard(ZSTD_error_srcSize_wrong));
    }

    #[test]
    fn errors_on_too_few_samples() {
        let samples: [&[u8]; 6] = [b"a", b"b", b"c", b"d", b"e", b"f"];

        assert!(!has_enough_samples_for_dictionary(samples.len()));
        let err = train_dictionary(&samples, 1024, 16).unwrap_err();
        assert_eq!(err, NxCompressionError::ZStandard(ZSTD_error_srcSize_wrong));
    }
}
