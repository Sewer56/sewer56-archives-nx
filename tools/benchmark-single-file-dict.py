#!/usr/bin/env python
import os
from pathlib import Path
from typing import NamedTuple
import subprocess
import shutil
from common import *

class BlockBenchmark(NamedTuple):
    name: str
    speed_with_dict: float    # MB/s
    speed_without_dict: float # MB/s
    speed_difference: float   # MB/s

def benchmark_block(compressed_file: Path, dictionary_path: Path = None) -> float:
    """Benchmark decompression of a single block using zstd."""
    cmd = ['zstd', '-b', '-i0', '-d']
    if dictionary_path:
        cmd.extend(['-D', str(dictionary_path)])
    cmd.append(str(compressed_file))
    
    result = subprocess.run(cmd, capture_output=True, text=True, check=True)
    return extract_speed(result.stdout)

def process_file(input_file: Path, fixed_dict_size: bool = False, manual_dict_size: int = None, block_size_kib: int = 128) -> list[BlockBenchmark]:
    """Process a single file and return its benchmark results."""
    print(f"\nProcessing {input_file}...")
    
    # Get appropriate dictionary size
    file_size = os.path.getsize(input_file)
    dict_size, dict_size_desc = get_dictionary_size(file_size, fixed_dict_size, manual_dict_size)
    print(f"Using dictionary size: {format_size(dict_size)} ({dict_size_desc})")
    
    # Create blocks and dictionary
    blocks_dir = split_file_into_blocks(input_file, block_size_kib)
    dict_file = train_dictionary(blocks_dir, dict_size)
    
    # Compress blocks both with and without dictionary
    compressed_with_dict = compress_blocks(blocks_dir, dict_file)
    compressed_without_dict = compress_blocks(blocks_dir, None)
    
    block_benchmarks = []
    
    # Get sorted lists of compressed files from both directories
    files_with_dict = sorted(compressed_with_dict.glob("*.zst"))
    files_without_dict = sorted(compressed_without_dict.glob("*.zst"))
    
    # Process matching pairs of files
    for with_dict_file, without_dict_file in zip(files_with_dict, files_without_dict):
        print(f"Benchmarking {with_dict_file.stem}...")
        
        # Benchmark with dictionary
        speed_with_dict = benchmark_block(with_dict_file, dict_file)
        print(f"  With dict: {speed_with_dict:.2f} MB/s")
        
        # Benchmark without dictionary
        speed_without_dict = benchmark_block(without_dict_file)
        print(f"  Without dict: {speed_without_dict:.2f} MB/s")
        
        # Calculate difference in speed
        speed_difference = speed_with_dict - speed_without_dict
        percent_difference = (speed_difference / speed_without_dict) * 100
        
        print(f"  Difference: {speed_difference:+.2f} MB/s ({percent_difference:+.1f}%)")
        
        benchmark = BlockBenchmark(
            name=with_dict_file.stem,
            speed_with_dict=speed_with_dict,
            speed_without_dict=speed_without_dict,
            speed_difference=speed_difference
        )
        block_benchmarks.append(benchmark)
    
    # Cleanup
    shutil.rmtree(blocks_dir)
    
    return block_benchmarks

def print_summary(file_benchmarks: dict[str, list[BlockBenchmark]]):
    """Print summary of all processed files."""
    print("\n=== Benchmark Summary ===")
    print(f"Files processed: {len(file_benchmarks)}")
    
    if not file_benchmarks:
        return
    
    for filename, benchmarks in file_benchmarks.items():
        avg_with_dict = sum(b.speed_with_dict for b in benchmarks) / len(benchmarks)
        avg_without_dict = sum(b.speed_without_dict for b in benchmarks) / len(benchmarks)
        avg_difference = sum(b.speed_difference for b in benchmarks) / len(benchmarks)
        percent_difference = (avg_difference / avg_without_dict) * 100
        
        print(f"\n{filename}:")
        print(f"  Average speed without dict: {avg_without_dict:.2f} MB/s")
        print(f"  Average speed with dict: {avg_with_dict:.2f} MB/s")
        print(f"  Average difference: {avg_difference:+.2f} MB/s ({percent_difference:+.1f}%)")
    
    # Overall statistics
    all_benchmarks = [b for benchmarks in file_benchmarks.values() for b in benchmarks]
    overall_avg_with_dict = sum(b.speed_with_dict for b in all_benchmarks) / len(all_benchmarks)
    overall_avg_without_dict = sum(b.speed_without_dict for b in all_benchmarks) / len(all_benchmarks)
    overall_avg_difference = sum(b.speed_difference for b in all_benchmarks) / len(all_benchmarks)
    overall_percent_difference = (overall_avg_difference / overall_avg_without_dict) * 100
    
    print("\nOverall averages:")
    print(f"  Without dictionary: {overall_avg_without_dict:.2f} MB/s")
    print(f"  With dictionary: {overall_avg_with_dict:.2f} MB/s")
    print(f"  Difference: {overall_avg_difference:+.2f} MB/s ({overall_percent_difference:+.1f}%)")

def main(input_dir: str, extension: str = None, fixed_dict_size: bool = False, manual_dict_size: int = None, block_size_kib: int = 128):
    """Main function to process all files in directory."""
    input_path = Path(input_dir)
    if not input_path.exists():
        raise FileNotFoundError(f"Input directory {input_dir} not found")
    
    # Process all files in directory
    file_benchmarks = {}
    pattern = f"*.{extension}" if extension else "*"
    
    print(f"Looking for files with pattern: {pattern}")
    print(f"Dictionary mode: {'fixed (110 KiB)' if fixed_dict_size else 'dynamic (1/100th of file size)'}")
    if manual_dict_size:
        print(f"Manual dictionary size: {format_size(manual_dict_size)}")
    
    for file_path in input_path.glob(pattern):
        if file_path.is_file() and not file_path.name.startswith('.'):
            try:
                benchmarks = process_file(file_path, fixed_dict_size, manual_dict_size, block_size_kib)
                file_benchmarks[file_path.name] = benchmarks
            except Exception as e:
                print(f"Error processing {file_path}: {e}")
    
    if not file_benchmarks:
        print(f"No matching files found in {input_dir}")
        return
        
    print_summary(file_benchmarks)

if __name__ == "__main__":
    import argparse
    
    parser = argparse.ArgumentParser(description="Benchmark ZSTD dictionary compression")
    parser.add_argument("input_dir", help="Directory containing files to process")
    parser.add_argument("--extension", "-e", 
                      help="File extension to process (e.g., 'txt' or 'json')")
    parser.add_argument("--fixed-dict-size", action="store_true",
                      help="Use fixed 110 KiB dictionary size instead of 1/100 of input size")
    parser.add_argument("--dict-size", type=int,
                      help="Manual override for dictionary size in bytes")
    parser.add_argument("--block-size", type=int, default=128,
                      help="Block size in KiB (default: 128)")
    
    args = parser.parse_args()
    main(args.input_dir, args.extension, args.fixed_dict_size, args.dict_size, args.block_size)