# Table of Contents (TOC)

This document describes the Table of Contents (TOC) format used in the archive files.

**Size**: 8 bytes

- `u1`: IsFlexibleFormat
- `u2`: Preset
- Remaining bits are allocated differently depending on the preset.

If the first bit is set, use [Flexible Entry Format 64 (FEF64)](#flexible-entry-format-64-fef64),
otherwise use one of the presets.

- `1XX` : [Flexible Entry Format 64 (FEF64)](#flexible-entry-format-64-fef64)
- `000` : [Preset `0`](#preset-0)
- `001` : [Preset `1`](#preset-1) [Preset0 w/o Hash]
- `010` : [Preset `2`](#preset-2) [Preset0 w/ 64-bit file sizes]
- `011` : [Preset `3`](#preset-3)

## Flexible Entry Format 64 (FEF64)

- **Summary**: A series of sub-formats with 16-byte `FileEntry`, fits most small mods and update packages.
- **Purpose**: Uploads/downloads of common mods to/from the internet.
    - Allows including/excluding the `Hash` field and supporting SOLID-less archives.
    - Most mods in practice will use this format.

Format:

- **TOC Header (8 Bytes)**:
    - `u1`: IsFlexibleFormat (Always `1`)
    - `u1`: HasHash
    - `u5`: CompressedPoolSizeBits (num bits for [CompressedPoolSize] in `Item Counts` below.)
    - `u5`: FileCountBits (num bits for [FileCount] in `Item Counts` below.)
    - `u5`: BlockCountBits (num bits for [BlockCount] in `Item Counts` below.)
    - `u5`: [DecompressedBlockOffset]Bits (num bits for [CompressedPoolSize] in `Item Counts` below.)
    - `u42`: Padding (`align8`) OR ItemCounts (if fits in 42 bits)
- [Optional: If Greater than 42 bits] **ItemCounts Struct (8 Bytes)**:
    - `align8` (Padding)
    - `u[CompressedPoolSizeBits]`: [CompressedPoolSize]
    - `u[BlockCountBits]`: [BlockCount]
    - `u[FileCountBits]`: [FileCount]
- **FileEntry** (8/16 bytes):
    - `u0` / `u64`: FileHash (XXH3) [Optional]
    - `u[64 - DecompressedBlockOffsetBits - FileCountBits - BlockCountBits]`: DecompressedSize
    - `u[DecompressedBlockOffsetBits]`: [DecompressedBlockOffset]
    - `u[FileCountBits]`: [FilePathIndex]
    - `u[BlockCountBits]`: [FirstBlockIndex]
- [Blocks[BlockCount]](#blocks)
    - `u29` CompressedBlockSize
    - `u3` [Compression]
- [StringPool]
    - `RawCompressedData...`

Values stored in `CompressedPoolSizeBits`, `BlockCountBits` and`FileCountBits` are offset by 1.
That means that if the stored value is `0`, we actually mean 1.

If ***CompressedPoolSizeBits + BlockCountBits + FileCountBits*** fit in the 42 bits of padding; then they
are placed in the lower 42 bits. Otherwise we allocate 8 bytes for the ItemCounts struct.

!!! note "Compressed pool data is ~4 bytes per entry usually."

    This version works under the assumption of ~21 bytes per entry,
    16 for entry, 5 for compressed file path. 4080 / 21 = 194.2 files.

## Preset 0

- **Summary**: 20-byte `FileEntry`. Suitable for 99.9% of mods.
- **Purpose**: General archival/unarchival of mods.
- **Limits:**
    - **Max File Count**: 256K
    - **Max Block Count**: 4M
    - **Max SOLID Block Size**: 16MiB
    - **Max File Size**: 4GiB
    - **Max Block Size**: 512MiB
- **Derived Limits**:
    - **Max Guaranteed Content Size**: 0.5PiB (4M blocks * 512MiB size)
    - **Max Size @64K Block Size**: 256GiB (4M blocks * 64KiB size)

Format:

- **TOC Header**:
    - `u1`: IsFlexibleFormat (Always `0`)
    - `u2`: Preset (Always `0`)
    - `u21`: [CompressedPoolSize]
    - `u22`: [BlockCount]
    - `u18`: [FileCount]
- **FileEntry** (20 bytes):
    - `u64`: FileHash (XXH3)
    - `u32`: DecompressedSize
    - `u24`: [DecompressedBlockOffset]
    - `u18`: [FilePathIndex]
    - `u22`: [FirstBlockIndex]
- [Blocks[BlockCount]](#blocks)
    - `u29` CompressedBlockSize
    - `u3` [Compression]
- [StringPool]
    - `RawCompressedData...`

## Preset 1

- **Summary**: 12-byte `FileEntry`. Variant of [Preset 0](#preset-0), but with no hash.
- **Purpose**: When hashes are not needed. e.g. Read-only virtual filesystems.
- **Limits:**
    - **Max File Count**: 256K
    - **Max Block Count**: 4M
    - **Max SOLID Block Size**: 16MiB
    - **Max File Size**: 4GiB
    - **Max Block Size**: 512MiB
- **Derived Limits**:
    - **Max Guaranteed Content Size**: 0.5PiB (4M blocks * 512MiB size)
    - **Max Size @64K Block Size**: 256GiB (4M blocks * 64KiB size)

Format:

- **TOC Header**:
    - `u1`: IsFlexibleFormat (Always `0`)
    - `u2`: Preset (Always `1`)
    - `u21`: [CompressedPoolSize]
    - `u22`: [BlockCount]
    - `u18`: [FileCount]
- **FileEntry** (12 bytes):
    - `u32`: DecompressedSize
    - `u24`: [DecompressedBlockOffset]
    - `u18`: [FilePathIndex]
    - `u22`: [FirstBlockIndex]
- [Blocks[BlockCount]](#blocks)
    - `u29` CompressedBlockSize
    - `u3` [Compression]
- [StringPool]
    - `RawCompressedData...`

## Preset 2

- **Summary**: 24-byte `FileEntry`. Variant of [Preset 0](#preset-0) with 64-bit file sizes.
- **Purpose**: Edge cases. Exceptionally huge archives.
- **Limits:**
    - **Max File Count**: 256K
    - **Max Block Count**: 4M
    - **Max SOLID Block Size**: 16MiB
    - **Max File Size**: 4GiB
    - **Max Block Size**: 512MiB
- **Derived Limits**:
    - **Max Guaranteed Content Size**: 0.5PiB (4M blocks * 512MiB size)
    - **Max Size @64K Block Size**: 256GiB (4M blocks * 64KiB size)
    - **Max Size @1M Block Size**: 4TiB (4M blocks * 1MiB size)

Format:

- **TOC Header**:
    - `u1`: IsFlexibleFormat (Always `0`)
    - `u2`: Preset (Always `2`)
    - `u21`: [CompressedPoolSize]
    - `u22`: [BlockCount]
    - `u18`: [FileCount]
- **FileEntry** (20 bytes):
    - `u64`: FileHash (XXH3)
    - `u64`: DecompressedSize
    - `u24`: [DecompressedBlockOffset]
    - `u18`: [FilePathIndex]
    - `u22`: [FirstBlockIndex]
- [Blocks[BlockCount]](#blocks)
    - `u29` CompressedBlockSize
    - `u3` [Compression]
- [StringPool]
    - `RawCompressedData...`

## Preset 3

- **Summary**: 8/16-byte `FileEntry`, for services hosting SOLID-less mods.
- **Purpose**: Uploads/downloads of mods with small files to the internet.
- **Limits:**
    - **Max File Count**: 64K
    - **Max Block Count**: 64K
    - **Max SOLID Block Size**: 0 MiB
    - **Max File Size**: 4GiB
    - **Max Block Size**: 512MiB
- **Derived Limits**:
    - **Max Guaranteed Content Size**: 32TiB (64K blocks * 512MiB block size)

Format:

- **TOC Header**:
    - `u1`: IsFlexibleFormat (Always `0`)
    - `u2`: Preset (Always `3`)
    - `u1`: HasHash (Always `0`)
    - `u20`: [CompressedPoolSize]
    - `u16`: [BlockCount]
    - `u16`: [FileCount]
    - `u8`: Padding (Align to 8 bytes)
- **FileEntry** (16 bytes):
    - `u64`: `FileHash` (XXH3)
    - `u32`: DecompressedSize
    - `u16`: [FilePathIndex]
    - `u16`: [FirstBlockIndex]
- [Blocks[BlockCount]](#blocks)
    - `u29` CompressedBlockSize
    - `u3` [Compression]
- [StringPool]
    - `RawCompressedData...`

## Field Explanations

### FileCount

!!! info "The `FileCount` in the TOC header determines the number of [FileEntry](#file-entries) structs following."

### BlockCount

!!! info "The `BlockCount` in the TOC header determines the number of [Block](#blocks) structs following."

### CompressedPoolSize

!!! info "The `CompressedPoolSize` in the TOC header specifies the size of the compressed [StringPool]"

Based on observation, a `CompressedPoolSize` of 16 MB can accommodate approximately 4.4 million
files with average path lengths.

### DecompressedBlockOffset

!!! info "Offset of the start of the file in the decompressed block"

### FilePathIndex

!!! info "The `FilePathIndex` specifies the order of the file path for a given `FileEntry` in the [StringPool]."

### FirstBlockIndex

!!! info "The `FirstBlockIndex` specifies the index of the block containing the file."

    Or the first block if the file is split into multiple chunks.

## File Entries

Use known fixed size and are word aligned to improve parsing speed; size 20-24 bytes per item depending on variant.

### Implicit Property: Chunk Count

!!! tip

    Files exceeding [Chunk Size](./File-Header.md#chunk-size) span multiple blocks.

Number of blocks used to store the file is calculated as: `DecompressedSize` / [Chunk Size](./File-Header.md#chunk-size),
and +1 if there is any remainder, i.e.

```csharp
public int GetChunkCount(int chunkSizeBytes)
{
    var count = DecompressedSize / (ulong)chunkSizeBytes;
    if (DecompressedSize % (ulong)chunkSizeBytes != 0)
        count += 1;

    return (int)count;
}
```

All chunk blocks are stored sequentially.

## Blocks

Each entry contains raw size of the block; and compression used. This avoids us having to have an offset for each block.

### Compression

Size: `3 bits` (0-7)

- `0`: Copy
- `1`: ZStandard
- `2`: LZ4
- `3`: BZip3

!!! note "As we do not store the length of the decompressed data, this must be determined from the compressed block."

!!! warning "Nx uses non-standard zstandard compressor settings"

    For more details, see [Stripping ZStandard Frame Headers]

## String Pool

The String Pool has the following format:

- `u32` DecompressedSize
- `u8[DecompressedSize]` RawData

The `DecompressedSize` stores the size of the pool after decompression, the compressed size can be
found above as [CompressedPoolSize].

The `RawData` is a buffer of UTF-8 deduplicated file path strings. Each string is null terminated.
The strings in this pool are first lexicographically sorted (to group similar paths together);
and then compressed using ZStd. This improves compression ratios.

For example a valid (decompressed) pool might look like this:
`data/textures/cat.png\0data/textures/dog.png`

String length is determined by searching null terminators. We will determine lengths of all strings ahead of time by scanning
for (`0x00`) using SIMD. No edge cases; `0x00` is guaranteed null terminator due to nature of UTF-8 encoding.

See UTF-8 encoding table:

|  Code point range  |  Byte 1  |  Byte 2  |  Byte 3  |  Byte 4  | Code points |
|:------------------:|:--------:|:--------:|:--------:|:--------:|:-----------:|
|  U+0000 - U+007F   | 0xxxxxxx |          |          |          |     128     |
|  U+0080 - U+07FF   | 110xxxxx | 10xxxxxx |          |          |    1920     |
|  U+0800 - U+FFFF   | 1110xxxx | 10xxxxxx | 10xxxxxx |          |    61440    |
| U+10000 - U+10FFFF | 11110xxx | 10xxxxxx | 10xxxxxx | 10xxxxxx |   1048576   |

When parsing the archive; we decode the StringPool into an array of strings.

!!! note "Nx archives should only use '/' as the path delimiter."

!!! tip "The number of items in the pool is equivalent to the number of files in the Table of Contents"

    If an archive has 1000 items, the pool has 1000 strings. The StringPool parser may only parse
    up to [FileCount] items. Files may choose not to use file names, in which case
    they would use a 0 length name. (Only `\0` null terminator)

!!! note

    It is possible to make ZSTD dictionaries for individual game directories that would further
    improve StringPool compression ratios.

    This might be added in the future but is currently not planned until additional testing and a
    backwards compatibility plan for decompressors missing the relevant dictionaries is decided.

## Performance Considerations

The header + TOC design aim to fit under 4096 bytes when possible. Based on a small 132 Mod, 7 Game Dataset, it is expected that >=90% of
mods out there will fit. This is to take advantage of read granularity; more specifically:

- **Page File Granularity**

For our use case where we memory map the file. Memory maps are always aligned to the page size, this is 4KiB on Windows and Linux (by default).
Therefore, a 5 KiB file will allocate 8 KiB and thus 3 KiB are wasted.

- **Unbuffered Disk Read**

If you have storage manufactured in the last 10 years, you probably have a physical sector size of 4096 bytes.

```pwsh
fsutil fsinfo ntfsinfo c:
# Bytes Per Physical Sector: 4096
```

a.k.a. ['Advanced Format'][Advanced-Format].
This is very convenient (especially since it matches page granularity); as when we open a mapped file (or even just read unbuffered),
we can read the exact amount of bytes to get header.

### Version Optimization Note

!!! info "The version formats are optimized around the read speeds of a 980 Pro NVMe SSD as reference"

  - 4K: 52us
  - 8K: 80us
  - 16K: 95us
  - 32K: 85us
  - 64K: 89us
  - 128K: 100us
  - 256K: 140us
  - 512K: 218us

There are 2 size 'thresholds':

- Up to 4K
- Up to 128K

In practice, the 128K size can handle around 5000 files when using a 20-byte [FileEntry].

This is considered to be the 'upper limit' in terms of file counts for mod packages; therefore
we do not have many variants beyond 20 bytes/entry. i.e. Beyond this limit, we don't aggressively optimize.

### Selecting CompressedPoolSize in Version/Variants

!!! tip "The [CompressedPoolSize] field size is proportional to the [FileCount]"

Consider the following:

- Game ***file names*** have a length of 16 bytes.
- 12 bytes if we exclude the extension.
- ***File paths*** are usually 5 bytes after compression.

To compensate for games with highly varying file names, ***we'll assume that the average
file path (after compression) is 8 bytes***.

***As a direct result, the [CompressedPoolSize] field should use 3 more bits than the [FileCount] field.***

[FileCount]: #filecount
[StringPool]: #string-pool
[Compression]: #compression
[FileEntry]: #file-entries
[Advanced-Format]: https://learn.microsoft.com/en-us/windows/win32/fileio/file-buffering#alignment-and-file-access-requirements
[CompressedPoolSize]: #compressedpoolsize
[fh-version]: ./File-Header.md#versionvariant
[BlockCount]: #blockcount
[DecompressedBlockOffset]: #decompressedblockoffset
[FilePathIndex]: #filepathindex
[FirstBlockIndex]: #firstblockindex
[Variant]: #variants