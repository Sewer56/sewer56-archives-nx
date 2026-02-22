!!! info "R3A supports dictionary compression, allowing you to improve compression ratios on small files."

!!! note "Optimization Note"

    Because the dictionary content itself is much greater than header size, no additional filtering/special
    tricks are done to reduce the size. Any files featuring a dictionary are likely going to easily shoot
    past the 4KiB sector limit. Instead we optimize more for decode speed.

## Data Structure

### Header

- **Dictionary Header (8 Bytes)**:
    - `u5`: Unused
    - `u4`: Version (Always 0)
    - `u27`: CompressedSize
    - `u28`: DecompressedSize

!!! note "A `CompressedSize` of 0 indicates data is not compressed, use the decompressed size instead"

### Payload

- **Payload Header (8 bytes)**:
    - `u11`: Unused
    - `u1`: HasHashes
    - `u8`: NumDictionaries (up to 254)
    - `u22`: NumMappings
    - `u22`: LastBlockIndexWithDictionary

- **Payload Data (Variable Size)**:
    - [BlockDictionaryIndex][NumMappings]
    - [BlockDictionaryLength][NumMappings]
    - `align32`
    - [DictionarySizes][NumDictionaries]
    - `if HasHashes == true`
        - `align64`
        - [DictionaryHashes][NumDictionaries]
    - RawDictionaryData

!!! info "Dictionary index `255` is reserved for 'no dictionary'"

#### BlockDictionaryIndex

!!! info "Assigns the index of the dictionary for this block mapping"

    This is an index into the dictionary sizes, dictionary hashes and raw dictionary data.

- `u8` DictionaryIndex

#### BlockDictionaryLength

!!! info "This represents the number of blocks corresponding to each [BlockDictionaryIndex]"

- `u8`: NumBlocks

!!! note "In most mods, number of consecutive blocks using same dictionary does not often exceed 256"

    And even if it does, the data would compress very well due to repeated bytes.

### Field Explanations

#### DictionarySizes

!!! info "An array of `u32` sizes for each dictionary."

#### RawDictionaryData

!!! info "This is raw dictionary data, length of each segment is extracted from [DictionarySizes]"

#### DictionaryHashes

!!! info "These are [XXH3] hashes of the dictionary content."

## Runtime Usage

The information will be used at runtime in the following manner:

1. Create an array where `block index` (u32) -> `dictionary index` (u8) is stored.
   So dictionary for block 0 is at index 0.
2. This array terminates at last block index where a dictionary is used.
3. If requested block index is out of range, assume no dictionary was used.

Because during packing files are usually sorted in size ascending order, and big files are usually
not using dictionaries, this means we will get fairly efficient memory usage.

## Reference Numbers

!!! info "For reference numbers, see [Research: Dictionaries] and [Research: Decode Speed]"

## Future Work

In the future there will be efforts to add `'standard'` dictionaries; that is, dictionaries which are
standardized across all R3A archives.

Plan is as follows:

- Allow user to specify a custom dictionary for a given extension/file group via API.
- That dictionary is embedded inside the R3A archive as normal.
- On load, dictionaries are hashed and deduplicated in memory in order to save RAM and improve caching efficiency.

This way, 'standardized' dictionaries can be used, without having any sort of centralized
authority over their index, location, etc.

[BlockDictionaryLength]: #blockdictionarylength
[DictionarySizes]: #dictionarysizes
[DictionaryHashes]: #dictionaryhashes
[BlockType]: #BlockType
[BlockDictionaryIndex]: #blockdictionaryindex
[Research: Dictionaries]: ../Research/DictionaryCompression.md
[Research: Decode Speed]: ../Research/DecodeSpeed.md
[XXH3]: https://xxhash.com/