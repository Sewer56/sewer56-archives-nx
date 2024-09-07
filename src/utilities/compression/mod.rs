// Compression modules
pub mod copy;

#[cfg(feature = "zstd")]
pub mod zstd;

#[cfg(feature = "lz4")]
pub mod lz4;

use crate::api::enums::compression_preference::CompressionPreference;
use copy::*;
use lz4::{Lz4CompressionError, Lz4DecompressionError};

/// A result type around compression functions..
/// Either a success code (number of bytes written), or an error code.
pub type CompressionResult = Result<usize, NxCompressionError>;

/// Represents an error returned from the Nx compression APIs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NxCompressionError {
    Copy(CopyCompressionError),
    ZStandard(ZSTD_ErrorCode),
    Lz4(Lz4CompressionError),
}

/// A result type around compression functions..
/// Either a success code (number of bytes decompressed), or an error code.
pub type DecompressionResult = Result<usize, NxDecompressionError>;

/// Represents an error returned from the Nx compression APIs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NxDecompressionError {
    Copy(CopyDecompressionError),
    ZStandard(ZSTD_ErrorCode),
    Lz4(Lz4DecompressionError),
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
        max_size = lz4::max_alloc_for_compress_size(source_length).max(max_size)
    }
    #[cfg(feature = "zstd")]
    {
        max_size = zstd::max_alloc_for_compress_size(source_length).max(max_size)
    }
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
        #[cfg(feature = "zstd")]
        CompressionPreference::ZStandard => zstd::compress(level, source, destination, used_copy),
        #[cfg(feature = "lz4")]
        CompressionPreference::Lz4 => lz4::compress(level, source, destination, used_copy),
        _ => panic!("Unsupported compression method"),
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
        #[cfg(feature = "zstd")]
        CompressionPreference::ZStandard => zstd::decompress(source, destination),
        #[cfg(feature = "lz4")]
        CompressionPreference::Lz4 => lz4::decompress(source, destination),
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
        #[cfg(feature = "zstd")]
        CompressionPreference::ZStandard => zstd::decompress_partial(source, destination),
        #[cfg(feature = "lz4")]
        CompressionPreference::Lz4 => lz4::decompress_partial(source, destination),
        _ => panic!("Unsupported partial decompression method"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::enums::compression_preference::CompressionPreference;
    use rstest::rstest;

    const TEST_DATA: &[u8] =
        b"This is compressible test data. testtesttesttesttesttesttesttesttesttesttesttest";
    const INCOMPRESSIBLE_DATA: &[u8] = b"thisdoenatcmpres"; // does not compress

    #[rstest]
    #[case::copy(CompressionPreference::Copy)]
    #[cfg_attr(feature = "zstd", case::zstd(CompressionPreference::ZStandard))]
    #[cfg_attr(feature = "lz4", case::lz4(CompressionPreference::Lz4))]
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
    #[cfg_attr(feature = "zstd", case::zstd(CompressionPreference::ZStandard))]
    #[cfg_attr(feature = "lz4", case::lz4(CompressionPreference::Lz4))]
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
    #[cfg_attr(
        feature = "zstd",
        case::zstd(
            CompressionPreference::ZStandard,
            NxCompressionError::Copy(CopyCompressionError::DestinationTooSmall) // ZStd delegates to copy, which then fails due to too small buffer.
        )
    )]
    #[cfg_attr(
        feature = "lz4",
        case::lz4(
            CompressionPreference::Lz4,
            NxCompressionError::Lz4(Lz4CompressionError::CompressionFailed)
        )
    )]
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
    #[cfg_attr(feature = "zstd", case::zstd(CompressionPreference::ZStandard))]
    #[cfg_attr(feature = "lz4", case::lz4(CompressionPreference::Lz4))]
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
    #[cfg_attr(
        feature = "zstd",
        case::zstd(
            CompressionPreference::ZStandard,
            NxDecompressionError::ZStandard(ZSTD_ErrorCode::ZSTD_error_dstSize_tooSmall)
        )
    )]
    #[cfg_attr(
        feature = "lz4",
        case::lz4(
            CompressionPreference::Lz4,
            NxDecompressionError::Lz4(Lz4DecompressionError::DecompressionFailed)
        )
    )]
    fn decompress_buffer_too_small_returms_error(
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
}
