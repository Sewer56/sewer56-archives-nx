// Compression modules
pub mod copy;
pub mod dictionary;
pub mod zstd;
pub mod zstd_stream;

#[cfg(feature = "lz4")]
pub mod lz4;

#[cfg(feature = "lz4")]
use lz4::*;

#[cfg(feature = "bzip3")]
pub mod bzip3;

#[cfg(feature = "bzip3")]
use bzip3::*;

use crate::api::enums::*;
use copy::*;
use thiserror_no_std::Error;

/// A result type around compression functions..
/// Either a success code (number of bytes written), or an error code.
pub type CompressionResult = Result<usize, NxCompressionError>;

/// Represents an error returned from the Nx compression APIs.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Error)]
pub enum NxCompressionError {
    #[error(transparent)]
    Copy(#[from] CopyCompressionError),
    #[error("ZStandard Error: {0:?}")]
    ZStandard(#[from] ZSTD_ErrorCode),
    #[cfg(feature = "lz4")]
    #[error(transparent)]
    Lz4(#[from] Lz4CompressionError),
    /// The LZ4 feature is not enabled. This can only ever be emitted if the error is disabled.
    #[cfg(not(feature = "lz4"))]
    #[error("LZ4 Feature not enabled")]
    Lz4NotEnabled,

    #[cfg(feature = "bzip3")]
    #[error(transparent)]
    Bzip3(#[from] Bzip3CompressionError),
    /// The BZip3 feature is not enabled. This can only ever be emitted if the error is disabled.
    #[cfg(not(feature = "bzip3"))]
    #[error("BZip3 Feature not enabled")]
    Bzip3NotEnabled,

    #[error("The operation was terminated during a stream operation with code: {0}")]
    TerminatedStream(usize),

    /// The LZMA feature is not currently supported.
    #[error("LZMA Feature not enabled")]
    LzmaNotEnabled,
}

/// A result type around compression functions..
/// Either a success code (number of bytes decompressed), or an error code.
pub type DecompressionResult = Result<usize, NxDecompressionError>;

/// Represents an error returned from the Nx compression APIs.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Error)]
pub enum NxDecompressionError {
    Copy(#[from] CopyDecompressionError),
    ZStandard(#[from] ZSTD_ErrorCode),
    #[cfg(feature = "lz4")]
    Lz4(#[from] Lz4DecompressionError),
    #[cfg(feature = "bzip3")]
    Bzip3(#[from] Bzip3DecompressionError),
}

/// Determines maximum memory needed to alloc to compress data with any method.
///
/// # Parameters
///
/// * `source_length`: Number of bytes at source.
pub fn max_alloc_for_compress_size(source_length: usize) -> usize {
    let mut max_size = copy::max_alloc_for_compress_size(source_length);
    #[cfg(feature = "lz4")]
    {
        max_size = lz4::max_alloc_for_compress_size(source_length).max(max_size);
    }
    #[cfg(feature = "bzip3")]
    {
        max_size = bzip3::max_alloc_for_compress_size(source_length).max(max_size);
    }

    max_size = zstd::max_alloc_for_compress_size(source_length).max(max_size);
    max_size
}

/// Compresses data with a specific method.
///
/// # Parameters
///
/// * `method`: Method we compress with.
/// * `level`: Level at which we are compressing.
/// * `source`: Source data to compress.
/// * `destination`: Destination buffer for compressed data.
/// * `used_copy`: If this is true, Copy compression was used, due to uncompressible data or by request.
///
/// # Returns
///
/// The number of bytes written to the destination.
pub fn compress(
    method: CompressionPreference,
    level: i32,
    source: &[u8],
    destination: &mut [u8],
    used_copy: &mut bool,
) -> CompressionResult {
    *used_copy = false;
    match method {
        CompressionPreference::Copy => copy::compress(source, destination, used_copy),
        CompressionPreference::ZStandard => zstd::compress(level, source, destination, used_copy),
        #[cfg(feature = "lz4")]
        CompressionPreference::Lz4 => lz4::compress(level, source, destination, used_copy),
        #[cfg(not(feature = "lz4"))]
        CompressionPreference::Lz4 => Err(NxCompressionError::Lz4NotEnabled),
        #[cfg(feature = "bzip3")]
        CompressionPreference::Bzip3 => bzip3::compress(source, destination, used_copy),
        #[cfg(not(feature = "bzip3"))]
        CompressionPreference::Bzip3 => Err(NxCompressionError::Bzip3NotEnabled),
        CompressionPreference::NoPreference => {
            zstd::compress(level, source, destination, used_copy)
        }
        CompressionPreference::LZMA => Err(NxCompressionError::LzmaNotEnabled),
    }
}

/// Compresses data with a specific method, with support for streaming and early termination.
///
/// # Parameters
///
/// * `method`: Method we compress with.
/// * `level`: Level at which we are compressing.
/// * `source`: Source data to compress.
/// * `destination`: Destination buffer for compressed data.
/// * `terminate_early`: Optional callback function that can terminate compression early.
/// * `used_copy`: If this is true, Copy compression was used, due to uncompressible data or by request.
///
/// # Returns
///
/// The number of bytes written to the destination.
pub fn compress_streamed<F>(
    method: CompressionPreference,
    level: i32,
    source: &[u8],
    destination: &mut [u8],
    terminate_early: Option<F>,
    used_copy: &mut bool,
) -> CompressionResult
where
    F: Fn() -> Option<usize>,
{
    *used_copy = false;
    match method {
        CompressionPreference::Copy => copy::compress(source, destination, used_copy),
        CompressionPreference::ZStandard => {
            zstd::compress_streamed(level, source, destination, terminate_early, used_copy)
        }
        #[cfg(feature = "lz4")]
        CompressionPreference::Lz4 => {
            lz4::compress_streamed(level, source, destination, terminate_early, used_copy)
        }
        #[cfg(not(feature = "lz4"))]
        CompressionPreference::Lz4 => Err(NxCompressionError::Lz4NotEnabled),
        #[cfg(feature = "bzip3")]
        CompressionPreference::Bzip3 => {
            bzip3::compress_streamed(source, destination, terminate_early, used_copy)
        }
        #[cfg(not(feature = "bzip3"))]
        CompressionPreference::Bzip3 => Err(NxCompressionError::Bzip3NotEnabled),
        CompressionPreference::NoPreference => {
            zstd::compress_streamed(level, source, destination, terminate_early, used_copy)
        }
        CompressionPreference::LZMA => Err(NxCompressionError::LzmaNotEnabled),
    }
}

/// Decompresses data with a specific method.
///
/// # Parameters
///
/// * `method`: Method we decompress with.
/// * `source`: Source data to decompress.
/// * `destination`: Destination buffer for decompressed data.
pub fn decompress(
    method: CompressionPreference,
    source: &[u8],
    destination: &mut [u8],
) -> DecompressionResult {
    match method {
        CompressionPreference::Copy => copy::decompress(source, destination),
        CompressionPreference::ZStandard => zstd::decompress(source, destination),
        #[cfg(feature = "lz4")]
        CompressionPreference::Lz4 => lz4::decompress(source, destination),
        #[cfg(feature = "bzip3")]
        CompressionPreference::Bzip3 => bzip3::decompress(source, destination),
        _ => panic!("Unsupported decompression method"),
    }
}

/// Partially decompresses data with a specific method until the destination buffer is filled.
///
/// # Parameters
///
/// * `method`: Method we decompress with.
/// * `source`: Source data to decompress.
/// * `destination`: Destination buffer for decompressed data.
pub fn decompress_partial(
    method: CompressionPreference,
    source: &[u8],
    destination: &mut [u8],
) -> DecompressionResult {
    match method {
        CompressionPreference::Copy => copy::decompress_partial(source, destination),
        CompressionPreference::ZStandard => zstd::decompress_partial(source, destination),
        #[cfg(feature = "lz4")]
        CompressionPreference::Lz4 => lz4::decompress_partial(source, destination),
        #[cfg(feature = "bzip3")]
        CompressionPreference::Bzip3 => bzip3::decompress_partial(source, destination),
        _ => panic!("Unsupported partial decompression method"), // TODO: Replace panic!
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use rstest::rstest;

    const TEST_DATA: &[u8] =
        b"This is compressible test data. testtesttesttesttesttesttesttesttesttesttesttest";
    const INCOMPRESSIBLE_DATA: &[u8] = b"thisdoenatcmpres"; // does not compress

    #[rstest]
    #[case::copy(CompressionPreference::Copy)]
    #[case::zstd(CompressionPreference::ZStandard)]
    #[cfg_attr(feature = "lz4", case::lz4(CompressionPreference::Lz4))]
    #[cfg_attr(feature = "bzip3", case::bzip3(CompressionPreference::Bzip3))]
    #[cfg_attr(miri, ignore)]
    fn can_round_trip(#[case] method: CompressionPreference) {
        let mut compressed = vec![0u8; max_alloc_for_compress_size(TEST_DATA.len())];
        let mut decompressed = vec![0u8; TEST_DATA.len()];
        let mut used_copy = false;

        let compressed_size =
            compress(method, 0, TEST_DATA, &mut compressed, &mut used_copy).unwrap();
        compressed.truncate(compressed_size);

        let decompressed_size = decompress(method, &compressed, &mut decompressed).unwrap();
        decompressed.truncate(decompressed_size);

        assert_eq!(TEST_DATA, decompressed.as_slice());
    }

    #[rstest]
    #[case::copy(CompressionPreference::Copy)]
    #[case::zstd(CompressionPreference::ZStandard)]
    #[cfg_attr(feature = "lz4", case::lz4(CompressionPreference::Lz4))]
    #[cfg_attr(feature = "bzip3", case::bzip3(CompressionPreference::Bzip3))]
    #[cfg_attr(miri, ignore)]
    fn incompressible_data_defaults_to_copy(#[case] method: CompressionPreference) {
        let mut compressed = vec![0u8; max_alloc_for_compress_size(INCOMPRESSIBLE_DATA.len())];
        let mut used_copy = false;

        let compressed_size = compress(
            method,
            0,
            INCOMPRESSIBLE_DATA,
            &mut compressed,
            &mut used_copy,
        )
        .unwrap();
        assert!(used_copy, "Incompressible data should use copy method");
        assert_eq!(compressed_size, INCOMPRESSIBLE_DATA.len());
    }

    #[rstest]
    #[case::copy(
        CompressionPreference::Copy,
        NxCompressionError::Copy(CopyCompressionError::DestinationTooSmall)
    )]
    #[case::zstd(
        CompressionPreference::ZStandard,
        NxCompressionError::Copy(CopyCompressionError::DestinationTooSmall) // ZStd delegates to copy, which then fails due to too small buffer.
    )]
    #[cfg_attr(
        feature = "lz4",
        case::lz4(
            CompressionPreference::Lz4,
            NxCompressionError::Lz4(Lz4CompressionError::CompressionFailed)
        )
    )]
    #[cfg_attr(
        feature = "bzip3",
        case::bzip3(
            CompressionPreference::Bzip3,
            NxCompressionError::Copy(CopyCompressionError::DestinationTooSmall) // BZip3 delegates to copy, which then fails due to too small buffer.
        )
    )]
    #[cfg_attr(miri, ignore)]
    fn destination_too_small_returns_err(
        #[case] method: CompressionPreference,
        #[case] expected_compression_error: NxCompressionError,
    ) {
        let small_buffer = [0u8; 10];
        let mut used_copy = false;

        // Test compression error
        let result = compress(
            method,
            0,
            TEST_DATA,
            &mut small_buffer.to_vec(),
            &mut used_copy,
        );

        assert_eq!(result.unwrap_err(), expected_compression_error);
    }

    #[rstest]
    #[case::copy(CompressionPreference::Copy)]
    #[case::zstd(CompressionPreference::ZStandard)]
    #[cfg_attr(feature = "lz4", case::lz4(CompressionPreference::Lz4))]
    #[cfg_attr(miri, ignore)]
    fn partial_decompression_succeeds(#[case] method: CompressionPreference) {
        let mut compressed = vec![0u8; max_alloc_for_compress_size(TEST_DATA.len())];
        let mut used_copy = false;

        let compressed_size =
            compress(method, 0, TEST_DATA, &mut compressed, &mut used_copy).unwrap();
        compressed.truncate(compressed_size);

        let mut half_decomp_data = vec![0u8; TEST_DATA.len() / 2];
        let decompressed_size =
            decompress_partial(method, &compressed, &mut half_decomp_data).unwrap();

        assert_eq!(
            decompressed_size,
            TEST_DATA.len() / 2,
            "Decompressed size should match the original data length"
        );
        assert_eq!(
            &half_decomp_data[..decompressed_size],
            &half_decomp_data[..TEST_DATA.len() / 2],
            "Decompressed data should match the original data"
        );
    }

    #[rstest]
    #[case::copy(
        CompressionPreference::Copy,
        NxDecompressionError::Copy(CopyDecompressionError::DestinationTooSmall)
    )]
    #[case::zstd(
        CompressionPreference::ZStandard,
        NxDecompressionError::ZStandard(ZSTD_ErrorCode::ZSTD_error_dstSize_tooSmall)
    )]
    #[cfg_attr(
        feature = "lz4",
        case::lz4(
            CompressionPreference::Lz4,
            NxDecompressionError::Lz4(Lz4DecompressionError::DecompressionFailed)
        )
    )]
    #[cfg_attr(
        feature = "bzip3",
        case::bzip3(
            CompressionPreference::Bzip3,
            NxDecompressionError::Bzip3(Bzip3CompressionError::CrcFailed)
        )
    )]
    #[cfg_attr(miri, ignore)]
    fn decompress_buffer_too_small_returns_error(
        #[case] method: CompressionPreference,
        #[case] expected_decompression_error: NxDecompressionError,
    ) {
        // Compress the test data
        let mut compressed = vec![0u8; max_alloc_for_compress_size(TEST_DATA.len())];
        let mut used_copy = false;
        let compressed_size =
            compress(method, 0, TEST_DATA, &mut compressed, &mut used_copy).unwrap();
        compressed.truncate(compressed_size);

        // Try to decompress with a buffer that's too small (half the size of the original data)
        let mut small_destination = vec![0u8; TEST_DATA.len() / 2];
        let result = decompress(method, &compressed, &mut small_destination);

        assert!(
            result.is_err(),
            "Should return an error when decompression buffer is too small"
        );
        assert_eq!(result.unwrap_err(), expected_decompression_error);
    }

    #[rstest]
    #[case::copy(CompressionPreference::Copy)]
    #[case::zstd(CompressionPreference::ZStandard)]
    #[cfg_attr(feature = "lz4", case::lz4(CompressionPreference::Lz4))]
    #[cfg_attr(miri, ignore)]
    fn can_round_trip_streamed(#[case] method: CompressionPreference) {
        let mut compressed = vec![0u8; max_alloc_for_compress_size(TEST_DATA.len())];
        let mut decompressed = vec![0u8; TEST_DATA.len()];
        let mut used_copy = false;

        let compressed_size = compress_streamed(
            method,
            0,
            TEST_DATA,
            &mut compressed,
            None::<fn() -> Option<usize>>,
            &mut used_copy,
        )
        .unwrap();
        compressed.truncate(compressed_size);

        let decompressed_size = decompress(method, &compressed, &mut decompressed).unwrap();
        decompressed.truncate(decompressed_size);
        assert_eq!(TEST_DATA, decompressed.as_slice());
    }

    #[rstest]
    #[case::zstd(CompressionPreference::ZStandard)]
    #[cfg_attr(feature = "lz4", case::lz4(CompressionPreference::Lz4))]
    #[cfg_attr(miri, ignore)]
    fn can_terminate_early_streamed(#[case] method: CompressionPreference) {
        let mut compressed = vec![0u8; max_alloc_for_compress_size(TEST_DATA.len())];
        let mut used_copy = false;
        let error_code = 0;

        let result = compress_streamed(
            method,
            0,
            TEST_DATA,
            &mut compressed,
            Some(|| Some(error_code)),
            &mut used_copy,
        );

        assert!(
            result.is_err(),
            "Early termination should complete successfully"
        );
        assert!(matches!(result, Err(NxCompressionError::TerminatedStream(x)) if x == error_code));
    }

    #[rstest]
    #[case::copy(CompressionPreference::Copy)]
    #[case::zstd(CompressionPreference::ZStandard)]
    #[cfg_attr(feature = "lz4", case::lz4(CompressionPreference::Lz4))]
    #[cfg_attr(miri, ignore)]
    fn incompressible_data_defaults_to_copy_streamed(#[case] method: CompressionPreference) {
        let mut compressed = vec![0u8; max_alloc_for_compress_size(INCOMPRESSIBLE_DATA.len())];
        let mut used_copy = false;

        let compressed_size = compress_streamed(
            method,
            0,
            INCOMPRESSIBLE_DATA,
            &mut compressed,
            None::<fn() -> Option<usize>>,
            &mut used_copy,
        )
        .unwrap();

        assert!(used_copy, "Incompressible data should use copy method");
        assert_eq!(compressed_size, INCOMPRESSIBLE_DATA.len());
    }

    #[rstest]
    #[case::copy(
        CompressionPreference::Copy,
        NxCompressionError::Copy(CopyCompressionError::DestinationTooSmall)
    )]
    #[case::zstd(
        CompressionPreference::ZStandard,
        NxCompressionError::Copy(CopyCompressionError::DestinationTooSmall)
    )]
    #[cfg_attr(
        feature = "lz4",
        case::lz4(
            CompressionPreference::Lz4,
            NxCompressionError::Lz4(Lz4CompressionError::CompressionFailed)
        )
    )]
    #[cfg_attr(miri, ignore)]
    fn destination_too_small_returns_err_streamed(
        #[case] method: CompressionPreference,
        #[case] expected_compression_error: NxCompressionError,
    ) {
        let small_buffer = [0u8; 10];
        let mut used_copy = false;

        let result = compress_streamed(
            method,
            0,
            TEST_DATA,
            &mut small_buffer.to_vec(),
            None::<fn() -> Option<usize>>,
            &mut used_copy,
        );

        assert_eq!(result.unwrap_err(), expected_compression_error);
    }
}
