#!/usr/bin/env python
import os
import glob
import shutil
import subprocess
from pathlib import Path
from typing import NamedTuple, Dict, Optional
from collections import defaultdict

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

def get_blocks_dir(input_file):
    """Create directory for blocks based on input filename."""
    input_path = Path(input_file)
    blocks_dir = input_path.parent / f"{input_path.stem}_blocks"
    
    # Remove directory completely if it exists
    if blocks_dir.exists():
        shutil.rmtree(blocks_dir)
    
    blocks_dir.mkdir(exist_ok=True)
    return blocks_dir

def split_file_into_blocks(input_file, block_size_kib=128):
    """Split a file into blocks of specified size in KiB."""
    block_size = block_size_kib * 1024  # Convert KiB to bytes
    input_path = Path(input_file)
    
    if not input_path.exists():
        raise FileNotFoundError(f"Input file {input_file} not found")
    
    # Create directory for blocks
    blocks_dir = get_blocks_dir(input_file)
    
    # Read and split the file
    with open(input_file, 'rb') as f:
        block_num = 0
        while True:
            block_data = f.read(block_size)
            if not block_data:
                break
                
            block_file = blocks_dir / f"block_{block_num:04d}"
            with open(block_file, 'wb') as block_f:
                block_f.write(block_data)
            block_num += 1
    
    return blocks_dir

def train_dictionary(blocks_dir, target_size):
    """Train a ZSTD dictionary using explicit block files list."""
    if not blocks_dir.exists():
        raise ValueError(f"Blocks directory {blocks_dir} not found")
    
    # Get sorted list of block files
    block_files = sorted(glob.glob(str(blocks_dir / "block_*")))
    if not block_files:
        raise ValueError("No block files found")
        
    output_dict = blocks_dir / "trained_dict"
    
    # Prepare zstd command with exact dictionary size
    cmd = [
        'zstd', '--train',
        *block_files,  # Expand list of files
        '-o', str(output_dict),
        '--maxdict', str(target_size),
        '--dictID', '1'
    ]
    
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        print(f"Dictionary training successful")
        
        # Compress the dictionary itself
        subprocess.run(['zstd', '-19', '-q', str(output_dict), '-o', str(output_dict) + '.zst'], check=True)
        
        actual_size = os.path.getsize(output_dict)
        if actual_size != target_size:
            print(f"Note: Dictionary size ({actual_size:,} bytes) differs from target ({target_size:,} bytes)")
        
        return output_dict
    except subprocess.CalledProcessError as e:
        print(f"Error training dictionary: {e.stderr}")
        raise

def compress_blocks(blocks_dir, dictionary_path=None, level=16):
    """Compress blocks with or without dictionary."""
    compressed_dir = blocks_dir / "compressed" / ("with_dict" if dictionary_path else "no_dict")
    compressed_dir.mkdir(parents=True, exist_ok=True)
    
    # Clean existing compressed files
    for f in compressed_dir.glob("*.zst"):
        f.unlink()
    
    block_files = sorted(blocks_dir.glob("block_*"))
    
    for block_file in block_files:
        output_file = compressed_dir / f"{block_file.name}.zst"
        
        cmd = ['zstd', '-q', f'-{level}']
        if dictionary_path:
            cmd.extend(['-D', str(dictionary_path)])
        cmd.extend(['-o', str(output_file), str(block_file)])
        
        subprocess.run(cmd, check=True)
    
    return compressed_dir

def format_size(size):
    """Format size in bytes to human readable format."""
    for unit in ['B', 'KiB', 'MiB', 'GiB']:
        if size < 1024.0:
            return f"{size:.2f} {unit}"
        size /= 1024.0
    return f"{size:.2f} TiB"

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

def get_dictionary_size(file_size: int, fixed_size: bool = False, manual_size: Optional[int] = None) -> tuple[int, str]:
    """Determine dictionary size and description based on input parameters."""
    if manual_size is not None:
        return manual_size, f"{format_size(manual_size)} (manual)"
    elif fixed_size:
        dict_size = 110 * 1024  # 110 KiB
        return dict_size, "110 KiB (fixed)"
    else:
        dict_size = file_size // 100  # Exactly 1/100th of input size
        return dict_size, "1/100 of input"

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