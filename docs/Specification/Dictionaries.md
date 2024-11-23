!!! info "Nx supports dictionary compression, allowing you to improve compression ratios on small files."

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

### Payload

!!! info "`NumBlocks` is the number of blocks in the Nx file."

- `u8`: NumDictionaries (up to 254)
- `u8`: NumMappings
- [BlockDictionaryIndex][NumMappings]
- [BlockMapping][NumMappings]
- `align32`
- [DictionarySizes][NumDictionaries]
- `align64`
- [DictionaryHashes][NumDictionaries]
- RawDictionaryData

!!! info "Dictionary index `255` is reserved for 'no dictionary'"

#### BlockDictionaryIndex

!!! info "Assigns the index of the dictionary for this block mapping"

- `u8` DictionaryIndex

#### BlockMapping

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

## Reference Numbers

!!! info "For reference numbers, see [Research: Dictionaries] and [Research: Decode Speed]"

## Future Work

In the future there will be efforts to add `'standard'` dictionaries; that is, dictionaries which are
standardized across all Nx archives.

Plan is as follows:

- Allow user to specify a custom dictionary for a given extension/file group via API.
- That dictionary is embedded inside the Nx archive as normal.
- On load, dictionaries are hashed and deduplicated in memory in order to save RAM and improve caching efficiency.

This way, 'standardized' dictionaries can be used, without having any sort of centralized
authority over their index, location, etc.

[BlockMapping]: #blockmapping
[DictionarySizes]: #dictionarysizes
[DictionaryHashes]: #dictionaryhashes
[BlockType]: #BlockType
[BlockDictionaryIndex]: #blockdictionaryindex
[Research: Dictionaries]: ../Research/DictionaryCompression.md
[Research: Decode Speed]: ../Research/DecodeSpeed.md
[XXH3]: https://xxhash.com/