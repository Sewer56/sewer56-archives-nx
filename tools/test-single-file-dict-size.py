#!/usr/bin/env python
import os
from pathlib import Path
from typing import NamedTuple, Dict, Optional
from common import *

class BlockStats(NamedTuple):
    original_size: int
    compressed_no_dict: int
    compressed_with_dict: int
    savings_no_dict: float
    savings_with_dict: float
    advantage: float
    bytes_saved: int  # Additional bytes saved with dictionary

class CompressionStats(NamedTuple):
    total_original: int
    total_compressed_no_dict: int
    total_compressed_with_dict: int
    dict_size: int
    compressed_dict_size: int
    savings_no_dict: float
    savings_with_dict: float
    difference: float
    total_size_with_dict: int
    block_stats: Dict[str, BlockStats]

def get_directory_size(directory: Path) -> int:
    """Calculate total size of all files in directory."""
    return sum(f.stat().st_size for f in directory.glob('**/*') if f.is_file())

def format_size_diff(size, ref_size):
    """Format size with difference percentage from reference."""
    percentage = (size / ref_size) * 100
    return f"{format_size(size)} ({percentage:.1f}%)"

def get_block_stats(blocks_dir: Path) -> Dict[str, BlockStats]:
    """Calculate compression statistics for each block."""
    stats = {}
    
    # Get all original blocks
    original_blocks = sorted(blocks_dir.glob("block_*"))
    compressed_no_dict = sorted((blocks_dir / "compressed" / "no_dict").glob("*.zst"))
    compressed_with_dict = sorted((blocks_dir / "compressed" / "with_dict").glob("*.zst"))
    
    for orig, no_dict, with_dict in zip(original_blocks, compressed_no_dict, compressed_with_dict):
        orig_size = orig.stat().st_size
        no_dict_size = no_dict.stat().st_size
        with_dict_size = with_dict.stat().st_size
        
        savings_no_dict = (1 - no_dict_size / orig_size) * 100
        savings_with_dict = (1 - with_dict_size / orig_size) * 100
        advantage = savings_with_dict - savings_no_dict
        bytes_saved = no_dict_size - with_dict_size
        
        stats[orig.name] = BlockStats(
            orig_size,
            no_dict_size,
            with_dict_size,
            savings_no_dict,
            savings_with_dict,
            advantage,
            bytes_saved
        )
    
    return stats

def compare_compression(blocks_dir, dict_path):
    """Compare compression results with and without dictionary."""
    original_size = get_directory_size(blocks_dir) - os.path.getsize(dict_path)  # Exclude dictionary size
    compressed_no_dict = get_directory_size(blocks_dir / "compressed" / "no_dict")
    compressed_with_dict = get_directory_size(blocks_dir / "compressed" / "with_dict")
    dict_size = os.path.getsize(dict_path)
    compressed_dict_size = os.path.getsize(str(dict_path) + '.zst')
    
    total_size_with_dict = compressed_with_dict + compressed_dict_size
    
    savings_no_dict = (1 - compressed_no_dict / original_size) * 100
    savings_with_dict = (1 - total_size_with_dict / original_size) * 100
    difference = savings_with_dict - savings_no_dict
    
    block_stats = get_block_stats(blocks_dir)
    
    return CompressionStats(
        original_size,
        compressed_no_dict,
        compressed_with_dict,
        dict_size,
        compressed_dict_size,
        savings_no_dict,
        savings_with_dict,
        difference,
        total_size_with_dict,
        block_stats
    )

def print_block_stats_summary(stats: CompressionStats):
    """Print summary statistics for blocks."""
    FOUR_KIB = 4 * 1024
    
    advantages = [s.advantage for s in stats.block_stats.values()]
    savings_no_dict = [s.savings_no_dict for s in stats.block_stats.values()]
    savings_with_dict = [s.savings_with_dict for s in stats.block_stats.values()]
    bytes_saved = [s.bytes_saved for s in stats.block_stats.values()]
    
    # Calculate 4KiB improvement metrics
    blocks_with_4kib_improvement = sum(1 for s in bytes_saved if s >= FOUR_KIB)
    percent_blocks_4kib = (blocks_with_4kib_improvement / len(stats.block_stats)) * 100
    
    # Calculate average 4KiB units saved
    avg_4kib_units = sum(b // FOUR_KIB for b in bytes_saved) / len(bytes_saved)
    
    print("\nPer-Block Statistics:")
    print(f"Number of blocks: {len(stats.block_stats)}")
    print(f"Blocks with ≥4KiB improvement: {blocks_with_4kib_improvement} ({percent_blocks_4kib:.1f}%)")
    print(f"Average 4KiB units saved per block: {avg_4kib_units:.2f}")
    
    print("\nDictionary Advantage (percentage points):")
    print(f"  Min: {min(advantages):.2f}%")
    print(f"  Max: {max(advantages):.2f}%")
    print(f"  Avg: {sum(advantages)/len(advantages):.2f}%")
    
    print("\nCompression Ratio:")
    print("  Without Dictionary:")
    print(f"    Min: {min(savings_no_dict):.2f}%")
    print(f"    Max: {max(savings_no_dict):.2f}%")
    print(f"    Avg: {sum(savings_no_dict)/len(savings_no_dict):.2f}%")
    
    print("  With Dictionary:")
    print(f"    Min: {min(savings_with_dict):.2f}%")
    print(f"    Max: {max(savings_with_dict):.2f}%")
    print(f"    Avg: {sum(savings_with_dict)/len(savings_with_dict):.2f}%")
    
    # Find best and worst blocks
    best_block = max(stats.block_stats.items(), key=lambda x: x[1].advantage)
    worst_block = min(stats.block_stats.items(), key=lambda x: x[1].advantage)
    
    print("\nMost Improved Block:")
    print(f"  {best_block[0]}:")
    print(f"    Original: {format_size(best_block[1].original_size)}")
    print(f"    Without dict: {format_size(best_block[1].compressed_no_dict)} ({best_block[1].savings_no_dict:.2f}% saved)")
    print(f"    With dict: {format_size(best_block[1].compressed_with_dict)} ({best_block[1].savings_with_dict:.2f}% saved)")
    print(f"    Advantage: {best_block[1].advantage:.2f}%")
    print(f"    Bytes saved: {format_size(best_block[1].bytes_saved)}")
    
    print("\nLeast Improved Block:")
    print(f"  {worst_block[0]}:")
    print(f"    Original: {format_size(worst_block[1].original_size)}")
    print(f"    Without dict: {format_size(worst_block[1].compressed_no_dict)} ({worst_block[1].savings_no_dict:.2f}% saved)")
    print(f"    With dict: {format_size(worst_block[1].compressed_with_dict)} ({worst_block[1].savings_with_dict:.2f}% saved)")
    print(f"    Advantage: {worst_block[1].advantage:.2f}%")
    print(f"    Bytes saved: {format_size(worst_block[1].bytes_saved)}")

def main(input_file, fixed_dict_size=None, block_size_kib=128, manual_dict_size=None):
    """Main function to handle the complete process."""
    file_size = os.path.getsize(input_file)
    
    dict_size, dict_size_desc = get_dictionary_size(
        file_size, 
        fixed_size=fixed_dict_size,
        manual_size=manual_dict_size
    )
    
    print(f"Processing {input_file}...")
    print(f"Input file size: {format_size(file_size)}")
    print(f"Target dictionary size: {format_size(dict_size)} ({dict_size_desc})")
    print(f"Splitting into {block_size_kib} KiB blocks...")
    
    blocks_dir = split_file_into_blocks(input_file, block_size_kib=block_size_kib)
    print(f"Created blocks in {blocks_dir}")
    
    print(f"Training dictionary...")
    dict_file = train_dictionary(blocks_dir, dict_size)
    print(f"Dictionary saved to: {dict_file}")
    
    print("\nCompressing blocks without dictionary...")
    compress_blocks(blocks_dir)
    
    print("Compressing blocks with dictionary...")
    compress_blocks(blocks_dir, dict_file)
    
    stats = compare_compression(blocks_dir, dict_file)
    
    print("\nOverall Size Comparison:")
    print(f"Original size:              {format_size(stats.total_original)}")
    print(f"Dictionary size:            {format_size_diff(stats.dict_size, stats.total_original)}")
    print(f"Dictionary compressed:      {format_size_diff(stats.compressed_dict_size, stats.total_original)}")
    print(f"Compressed (no dict):       {format_size_diff(stats.total_compressed_no_dict, stats.total_original)}")
    print(f"Compressed (with dict):     {format_size_diff(stats.total_compressed_with_dict, stats.total_original)}")
    print(f"Total with dict+blocks:     {format_size_diff(stats.total_size_with_dict, stats.total_original)}")

    print(f"\nOverall Space Savings:")
    print(f"Without dictionary:         {stats.savings_no_dict:.2f}%")
    print(f"With dictionary:            {stats.savings_with_dict:.2f}%")
    print(f"Dictionary advantage:       {stats.difference:.2f}%")
    
    print_block_stats_summary(stats)
    
    return dict_file

if __name__ == "__main__":
    import argparse
    
    parser = argparse.ArgumentParser(description="Train ZSTD dictionary and compare compression")
    parser.add_argument("input_file", help="Input file to process")
    parser.add_argument("--fixed-dict-size", action="store_true", 
                      help="Use fixed 110 KiB dictionary size instead of 1/100 of input size")
    parser.add_argument("--block-size", type=int, default=128,
                      help="Block size in KiB (default: 128)")
    parser.add_argument("--dict-size", type=int,
                      help="Manual override for dictionary size in bytes")
    
    args = parser.parse_args()
    main(args.input_file, 
         fixed_dict_size=args.fixed_dict_size, 
         block_size_kib=args.block_size,
         manual_dict_size=args.dict_size)