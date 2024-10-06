# Table of Contents (TOC)

This document describes the Table of Contents (TOC) format used in the archive files.

**Size**: 8 bytes

- `u3`: Version (`0-7`)
- Remaining bits are allocated differently depending on the version.

## Version `0`

- **Summary**: 20-byte `FileEntry`. Suitable for 99.9% of mods.
- **Purpose**: General archival/unarchival of larger mods.
- **Limits:**
    - **Max File Count**: 1M
    - **Max Block Count**: 256K
    - **Max Block Size**: 64MiB
    - **Max Content Size**: 16,384 TiB
    - **Max File Size**: 4GiB

Format:

- **TOC Header**:
    - `u3`: Version (`0`)
    - `u23`: [StringPoolSize]
    - `u18`: [BlockCount]
    - `u20`: [FileCount]
- **FileEntry** (20 bytes):
    - `u64`: FileHash (XXH3)
    - `u32`: DecompressedSize
    - `u26`: DecompressedBlockOffset
    - `u20`: [FilePathIndex]
    - `u18`: [FirstBlockIndex]
- [Blocks[BlockCount]](#blocks)
    - `u29` CompressedBlockSize
    - `u3` [Compression]
- [StringPool]
    - `RawCompressedData...`

## Version `1`

- **Summary**: 24-byte `FileEntry`. For truly exceptional edge cases.
- **Purpose**: Edge cases. Exceptionally huge archives.
- **Limits:**
    - **Max File Count**: 1M
    - **Max Block Count**: 16,384 G
    - **Max Block Size**: 64MiB
    - **Max Content Size**: 1,073,741,824 TiB
    - **Max File Size**: 256GiB

Format:

- **TOC Header**:
    - `u3`: Version (`1`)
    - `u23`: [StringPoolSize]
    - `u18`: [BlockCount]
    - `u20`: [FileCount]
- **FileEntry** (24 bytes):
    - `u64`: FileHash (XXH3)
    - `u38`: DecompressedSize
    - `u26`: DecompressedBlockOffset
    - `u20`: [FilePathIndex]
    - `u44`: [FirstBlockIndex]
- [Blocks[BlockCount]](#blocks)
    - `u29` CompressedBlockSize
    - `u3` [Compression]
- [StringPool]
    - `RawCompressedData...`


## Version `2`

- **Summary**: 12-byte `FileEntry`. Variant of [Version 1](#version-1), but with no hash, reduced file count and increased block count.
- **Purpose**: When hashes are not needed. e.g. Read-only virtual filesystems.
- **Limits:**
    - **Max File Count**: 256K
    - **Max Block Count**: 1M
    - **Max Block Size**: 64MiB
    - **Max Content Size**: 16,384 TiB
    - **Max File Size**: 4GiB
    - **Max Size for VFS:**: 64GiB (@ 64K Block Sizes)

Format:

- **TOC Header**:
    - `u3`: Version (`2`)
    - `u23`: [StringPoolSize]
    - `u20`: [BlockCount]
    - `u18`: [FileCount]
- **FileEntry** (12 bytes):
    - `u32`: DecompressedSize
    - `u26`: DecompressedBlockOffset
    - `u18`: [FilePathIndex]
    - `u20`: [FirstBlockIndex]
- [Blocks[BlockCount]](#blocks)
    - `u29` CompressedBlockSize
    - `u3` [Compression]
- [StringPool]
    - `RawCompressedData...`

!!! note "Compressed pool data is ~4 bytes per entry."

    This version works under the assumption of ~24 bytes per entry.
    To reach the 4K size target, `4080 / 20 == 255` files.

## Version `3`

- **Summary**: 16-byte `FileEntry`, fits most small mods and update packages.
- **Purpose**: Uploads/downloads to/from the internet.
- **Limits:**
    - **Max File Count**: 255
    - **Max Block Count**: 255
    - **Max Block Size**: 1MiB
    - **Max Content Size**: 255MiB
    - **Max File Size**: 255MiB

Format:

- **TOC Header**:
    - `u3`: Version (`3`)
    - `u28`: [StringPoolSize]
    - `u8`: [BlockCount]
    - `u8`: [FileCount]
    - `u17`: Padding
- **FileEntry** (16 bytes):
    - `u64`: `FileHash` (XXH3)
    - `u28`: DecompressedSize
    - `u20`: DecompressedBlockOffset
    - `u8`: [FilePathIndex]
    - `u8`: [FirstBlockIndex]
- [Blocks[BlockCount]](#blocks)
    - `u29` CompressedBlockSize
    - `u3` [Compression]
- [StringPool]
    - `RawCompressedData...`

!!! note "Compressed pool data is ~4 bytes per entry usually."

    This version works under the assumption of ~24 bytes per entry.
    To reach the 4K size target, `4080 / 24 == 204` files.

## Version `7`

- **Purpose**: **RESERVED** for extended formats.

## Field Explanations

### FileCount

!!! info "The `FileCount` in the TOC header determines the number of [FileEntry](#file-entries) structs following."

### BlockCount

!!! info "The `BlockCount` in the TOC header determines the number of [Block](#blocks) structs following."

### StringPoolSize

!!! info "The `StringPoolSize` in the TOC header specifies the size of the compressed [StringPool]"

Based on observation, a `StringPoolSize` of 16 MB can accommodate approximately 4.4 million
files with average path lengths.

### DecompressedBlockOffset

!!! info "Offset of the decompressed block"

### FilePathIndex

!!! info "The `FilePathIndex` specifies the order of the file path for a given `FileEntry` in the [StringPool]."

### FirstBlockIndex

!!! info "The `FirstBlockIndex` specifies the index of the block containing the file."

    Or the first block if the file is split into multiple chunks.

## File Entries

Use known fixed size and are 4 byte aligned to improve parsing speed; size 20-24 bytes per item depending on variant.

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
- `3-7`: Reserved

!!! note "As we do not store the length of the decompressed data, this must be determined from the compressed block."

## String Pool

!!! note "Nx archives should only use '/' as the path delimiter."

Raw buffer of UTF-8 deduplicated strings of file paths. Each string is null terminated.
The strings in this pool are first lexicographically sorted (to group similar paths together); and then compressed using ZStd.
As for decompression, size of this pool is unknown until after decompression is done; file header should specify sufficient buffer size.

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

!!! tip "The number of items in the pool is equivalent to the number of files in the Table of Contents"

    If an archive has 1000 items, the pool has 1000 strings.

!!! note

    It is possible to make ZSTD dictionaries for individual game directories that would further improve StringPool compression ratios.

    This might be added in the future but is currently not planned until additional testing and a backwards compatibility
    plan for decompressors missing the relevant dictionaries is decided.

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

[FileCount]: #filecount
[StringPool]: #string-pool
[Compression]: #compression
[FileEntry]: #file-entries
[Advanced-Format]: https://learn.microsoft.com/en-us/windows/win32/fileio/file-buffering#alignment-and-file-access-requirements
[StringPoolSize]: #stringpoolsize
[fh-version]: ./File-Header.md#versionvariant
[BlockCount]: #blockcount
[FilePathIndex]: #filepathindex
[FirstBlockIndex]: #firstblockindex