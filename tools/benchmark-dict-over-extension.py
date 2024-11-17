#!/usr/bin/env python
from pathlib import Path
from typing import NamedTuple, List
import subprocess
import shutil
from dataclasses import dataclass
from common import *

@dataclass
class SmallFile:
    path: Path
    size: int

class Block(NamedTuple):
    files: List[SmallFile]
    total_size: int
    block_id: int

class CompressionResult(NamedTuple):
    size: int
    speed: float

class BlockResult(NamedTuple):
    block_id: int
    num_files: int
    total_size: int
    individual_with_dict: CompressionResult
    individual_no_dict: CompressionResult
    solid_with_dict: CompressionResult
    solid_no_dict: CompressionResult

def trim_file(file_path: Path, block_size: int, output_dir: Path) -> Path:
    """Trim a file to block_size bytes and save to output directory."""
    trimmed_path = output_dir / f"{file_path.name}.trimmed"
    with open(file_path, 'rb') as infile:
        with open(trimmed_path, 'wb') as outfile:
            outfile.write(infile.read(block_size))
    return trimmed_path

def find_files(input_dir: Path, extension: str, size_limit: int, mode: str) -> List[SmallFile]:
    """Find files based on size criteria and mode.
    
    Args:
        mode: Either 'small' for files under size_limit or 'trim' for files over size_limit
    """
    files = []
    pattern = f"**/*.{extension}" if extension else "**/*"
    
    for file_path in input_dir.glob(pattern):
        if file_path.is_file():
            size = file_path.stat().st_size
            if (mode == 'small' and size <= size_limit) or \
               (mode == 'trim' and size >= size_limit):
                files.append(SmallFile(file_path, size))
    
    return sorted(files, key=lambda x: x.size)

def create_blocks(files: List[SmallFile], block_size: int, mode: str) -> List[Block]:
    """Create blocks based on mode.
    
    Args:
        mode: Either 'small' for variable-sized blocks or 'trim' for fixed-size blocks
    """
    if mode == 'trim':
        return [Block([file], block_size, i) for i, file in enumerate(files)]
    
    # For small files mode, arrange into blocks of approximately block_size
    blocks = []
    current_block = []
    current_size = 0
    block_id = 0
    
    for file in files:
        if current_block and current_size + file.size > block_size:
            blocks.append(Block(current_block, current_size, block_id))
            current_block = []
            current_size = 0
            block_id += 1
        
        current_block.append(file)
        current_size += file.size
    
    if current_block:
        blocks.append(Block(current_block, current_size, block_id))
    
    return blocks

def compress_files_individually(files: List[SmallFile], output_dir: Path, block_size: int = None,
                              dictionary_path: Path = None, mode: str = 'small') -> CompressionResult:
    """Compress files individually with optional trimming."""
    total_size = 0
    total_speed = 0
    
    # For trim mode, create trimmed versions first
    if mode == 'trim':
        trimmed_dir = output_dir / "trimmed"
        trimmed_dir.mkdir()
        process_files = [trim_file(f.path, block_size, trimmed_dir) for f in files]
    else:
        process_files = [f.path for f in files]
    
    for file in process_files:
        output_file = output_dir / f"{file.name}.zst"
        
        cmd = ['zstd', '-12', '-q']
        if dictionary_path:
            cmd.extend(['-D', str(dictionary_path)])
        cmd.extend(['-o', str(output_file), str(file)])
        
        subprocess.run(cmd, check=True)
        total_size += output_file.stat().st_size
        
        # Benchmark decompression
        speed = benchmark_file(output_file, dictionary_path)
        if mode == 'trim':
            total_speed += speed * block_size
        else:
            file_size = files[process_files.index(file)].size
            total_speed += speed * file_size
    
    # Cleanup trimmed files if necessary
    if mode == 'trim':
        shutil.rmtree(trimmed_dir)
        total_bytes = len(files) * block_size
    else:
        total_bytes = sum(f.size for f in files)
    
    avg_speed = total_speed / total_bytes
    return CompressionResult(total_size, avg_speed)

def compress_block(files: List[SmallFile], output_file: Path, block_size: int = None,
                  dictionary_path: Path = None, mode: str = 'small') -> CompressionResult:
    """Compress files as a single block with optional trimming."""
    temp_dir = output_file.parent / "temp"
    temp_dir.mkdir()
    temp_file = temp_dir / "concatenated"
    
    with open(temp_file, 'wb') as outfile:
        for file in files:
            with open(file.path, 'rb') as infile:
                if mode == 'trim':
                    outfile.write(infile.read(block_size))
                else:
                    outfile.write(infile.read())
    
    cmd = ['zstd', '-12', '-q']
    if dictionary_path:
        cmd.extend(['-D', str(dictionary_path)])
    cmd.extend(['-o', str(output_file), str(temp_file)])
    
    subprocess.run(cmd, check=True)
    compressed_size = output_file.stat().st_size
    
    # Benchmark decompression
    speed = benchmark_file(output_file, dictionary_path)
    
    # Cleanup
    shutil.rmtree(temp_dir)
    
    return CompressionResult(compressed_size, speed)

def prepare_training_samples(files: List[SmallFile], work_dir: Path,
                           block_size: int = None, mode: str = 'small') -> Path:
    """Prepare directory with files for dictionary training."""
    samples_dir = work_dir / "training_samples"
    samples_dir.mkdir()
    
    if mode == 'trim':
        for file in files:
            trim_file(file.path, block_size, samples_dir)
    else:
        for file in files:
            shutil.copy2(file.path, samples_dir)
    
    return samples_dir

def process_blocks(blocks: List[Block], dict_size: int = 110 * 1024,
                  block_size: int = None, mode: str = 'small') -> List[BlockResult]:
    """Process blocks with all compression methods."""
    if not blocks:
        return []
    
    print(f"\nProcessing {len(blocks)} blocks...")
    total_size = sum(block.total_size for block in blocks)
    print(f"Total size: {format_size(total_size)}")
    print(f"Dictionary size: {format_size(dict_size)}")
    if mode == 'trim':
        print(f"Block size: {format_size(block_size)}")
    
    # Create temporary work directory
    work_dir = Path("temp_work_dir")
    if work_dir.exists():
        shutil.rmtree(work_dir)
    work_dir.mkdir()
    
    # Get all files for dictionary training
    all_files = [f for block in blocks for f in block.files]
    
    # Prepare samples and train dictionary
    samples_dir = prepare_training_samples(all_files, work_dir, block_size, mode)
    dict_file = work_dir / "trained_dict"
    
    sample_files = list(samples_dir.iterdir())
    cmd = ['zstd', '--train', *[str(f) for f in sample_files],
           '-o', str(dict_file), '--maxdict', str(dict_size), '--dictID', '1']
    subprocess.run(cmd, capture_output=True, text=True, check=True)
    
    total_training_size = sum(f.stat().st_size for f in sample_files)
    print(f"Dictionary trained on {len(sample_files)} files (total {format_size(total_training_size)})")
    print(f"Actual dictionary size: {format_size(dict_file.stat().st_size)}")
    
    results = []
    for block in blocks:
        print(f"\nProcessing block {block.block_id} ({len(block.files)} files, {format_size(block.total_size)})")
        
        # Create directories for individual compression
        individual_with_dict_dir = work_dir / f"block_{block.block_id}_individual_with_dict"
        individual_no_dict_dir = work_dir / f"block_{block.block_id}_individual_no_dict"
        individual_with_dict_dir.mkdir()
        individual_no_dict_dir.mkdir()
        
        # Process with all methods
        individual_with_dict = compress_files_individually(
            block.files, individual_with_dict_dir, block_size, dict_file, mode)
        individual_no_dict = compress_files_individually(
            block.files, individual_no_dict_dir, block_size, mode=mode)
        
        solid_with_dict_file = work_dir / f"block_{block.block_id}_solid_with_dict.zst"
        solid_no_dict_file = work_dir / f"block_{block.block_id}_solid_no_dict.zst"
        solid_with_dict = compress_block(
            block.files, solid_with_dict_file, block_size, dict_file, mode)
        solid_no_dict = compress_block(
            block.files, solid_no_dict_file, block_size, mode=mode)
        
        print(f"  Individual (no dict):   {format_size(individual_no_dict.size)} ({individual_no_dict.speed:.2f} MB/s)")
        print(f"  Individual (with dict):  {format_size(individual_with_dict.size)} ({individual_with_dict.speed:.2f} MB/s)")
        print(f"  Solid block (no dict):   {format_size(solid_no_dict.size)} ({solid_no_dict.speed:.2f} MB/s)")
        print(f"  Solid block (with dict): {format_size(solid_with_dict.size)} ({solid_with_dict.speed:.2f} MB/s)")
        
        results.append(BlockResult(
            block_id=block.block_id,
            num_files=len(block.files),
            total_size=block.total_size,
            individual_with_dict=individual_with_dict,
            individual_no_dict=individual_no_dict,
            solid_with_dict=solid_with_dict,
            solid_no_dict=solid_no_dict
        ))
    
    # Cleanup
    shutil.rmtree(work_dir)
    return results

def print_summary(results: List[BlockResult]):
    """Print summary of all compression methods."""
    if not results:
        return
    
    total_original = sum(r.total_size for r in results)
    total_individual_no_dict = sum(r.individual_no_dict.size for r in results)
    total_individual_with_dict = sum(r.individual_with_dict.size for r in results)
    total_solid_no_dict = sum(r.solid_no_dict.size for r in results)
    total_solid_with_dict = sum(r.solid_with_dict.size for r in results)
    
    # Calculate weighted average speeds
    avg_speed_individual_no_dict = sum(r.individual_no_dict.speed * r.total_size for r in results) / total_original
    avg_speed_individual_with_dict = sum(r.individual_with_dict.speed * r.total_size for r in results) / total_original
    avg_speed_solid_no_dict = sum(r.solid_no_dict.speed * r.total_size for r in results) / total_original
    avg_speed_solid_with_dict = sum(r.solid_with_dict.speed * r.total_size for r in results) / total_original
    
    print("\n=== Compression Summary ===")
    print(f"Total original size: {format_size(total_original)}")
    print("\nCompressed sizes:")
    print(f"  Individual files (no dict):   {format_size(total_individual_no_dict)} ({total_original/total_individual_no_dict:.2f}x)")
    print(f"  Individual files (with dict):  {format_size(total_individual_with_dict)} ({total_original/total_individual_with_dict:.2f}x)")
    print(f"  Solid blocks (no dict):        {format_size(total_solid_no_dict)} ({total_original/total_solid_no_dict:.2f}x)")
    print(f"  Solid blocks (with dict):      {format_size(total_solid_with_dict)} ({total_original/total_solid_with_dict:.2f}x)")
    
    print("\nSpace savings vs individual (no dict):")
    dict_savings = total_individual_no_dict - total_individual_with_dict
    solid_savings = total_individual_no_dict - total_solid_no_dict
    solid_dict_savings = total_individual_no_dict - total_solid_with_dict
    print(f"  Dictionary advantage:         {format_size(dict_savings)} ({(dict_savings/total_individual_no_dict)*100:.1f}%)")
    print(f"  Solid block advantage:        {format_size(solid_savings)} ({(solid_savings/total_individual_no_dict)*100:.1f}%)")
    print(f"  Solid block + dict advantage: {format_size(solid_dict_savings)} ({(solid_dict_savings/total_individual_no_dict)*100:.1f}%)")
    
    print("\nAverage decompression speeds:")
    print(f"  Individual files (no dict):   {avg_speed_individual_no_dict:.2f} MB/s")
    print(f"  Individual files (with dict):  {avg_speed_individual_with_dict:.2f} MB/s")
    print(f"  Solid blocks (no dict):        {avg_speed_solid_no_dict:.2f} MB/s")
    print(f"  Solid blocks (with dict):      {avg_speed_solid_with_dict:.2f} MB/s")

def main(input_dir: str, extension: str, mode: str = 'small',
         block_size: int = 64 * 1024, dict_size: int = 110 * 1024):
    """Main function to process files in directory."""
    input_path = Path(input_dir)
    if not input_path.exists():
        raise FileNotFoundError(f"Input directory {input_dir} not found")
    
    if mode == 'small':
        print(f"Looking for .{extension} files under {format_size(block_size)}...")
        print(f"Target block size: {format_size(block_size)}")
    else:  # trim mode
        print(f"Looking for .{extension} files at least {format_size(block_size)} in size...")
        print(f"Will trim all files to exactly {format_size(block_size)}")
    
    files = find_files(input_path, extension, block_size, mode)
    
    if not files:
        print(f"No matching files found in {input_dir}")
        return
    
    print(f"Found {len(files)} files")
    
    blocks = create_blocks(files, block_size, mode)
    print(f"{'Arranged' if mode == 'small' else 'Created'} into {len(blocks)} blocks")
    
    results = process_blocks(blocks, dict_size, block_size, mode)
    print_summary(results)

if __name__ == "__main__":
    import argparse
    
    parser = argparse.ArgumentParser(description="Benchmark ZSTD compression methods")
    parser.add_argument("input_dir", help="Directory containing files to process")
    parser.add_argument("--extension", "-e", required=True,
                    help="File extension to process (e.g., 'txt' or 'json')")
    parser.add_argument("--mode", choices=['small', 'trim'], default='small',
                    help="Processing mode: 'small' for files under size limit, 'trim' for larger files")
    parser.add_argument("--block-size", type=int, default=64 * 1024,
                    help="Block size in bytes (default: 64KB). In 'small' mode, this is the target size for blocks. "
                        "In 'trim' mode, files are trimmed to exactly this size.")
    parser.add_argument("--dict-size", type=int, default=110 * 1024,
                    help="Dictionary size in bytes (default: 110KB)")
    
    args = parser.parse_args()
    main(args.input_dir, args.extension, args.mode, args.block_size, args.dict_size)