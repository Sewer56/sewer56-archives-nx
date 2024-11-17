#!/usr/bin/env python
import os
import glob
import shutil
import subprocess
from pathlib import Path
import re

def get_blocks_dir(input_file):
    """Create directory for blocks based on input filename."""
    input_path = Path(input_file)
    blocks_dir = input_path.parent / f"{input_path.stem}_blocks"
    
    if blocks_dir.exists():
        shutil.rmtree(blocks_dir)
    
    blocks_dir.mkdir(exist_ok=True)
    return blocks_dir

def split_file_into_blocks(input_file, block_size_kib=128):
    """Split a file into blocks of specified size in KiB."""
    block_size = block_size_kib * 1024
    input_path = Path(input_file)
    
    if not input_path.exists():
        raise FileNotFoundError(f"Input file {input_file} not found")
    
    blocks_dir = get_blocks_dir(input_file)
    
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

def train_dictionary(blocks_dir, target_size, level=12):
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
        '--dictID', '1',
        f'-{level}'
    ]
    
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        
        # Compress the dictionary itself
        subprocess.run(['zstd', '-19', '-q', str(output_dict), '-o', str(output_dict) + '.zst'], check=True)
        
        actual_size = os.path.getsize(output_dict)
        if actual_size != target_size:
            print(f"Note: Dictionary size ({actual_size:,} bytes) differs from target ({target_size:,} bytes)")
        
        return output_dict
    except subprocess.CalledProcessError as e:
        print(f"Error training dictionary: {e.stderr}")
        raise

def compress_blocks(blocks_dir, dictionary_path=None, level=12):
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

def get_dictionary_size(file_size: int, fixed_size: bool = False, manual_size: int = None) -> tuple[int, str]:
    """Determine dictionary size and description based on input parameters."""
    if manual_size is not None:
        return manual_size, f"{format_size(manual_size)} (manual)"
    elif fixed_size:
        dict_size = 110 * 1024  # 110 KiB
        return dict_size, "110 KiB (fixed)"
    else:
        dict_size = file_size // 100  # Exactly 1/100th of input size
        return dict_size, "1/100 of input"

def extract_speed(output: str) -> float:
    """Extract the last decompression speed from zstd benchmark output."""
    matches = list(re.finditer(r'(\d+\.\d+)\s*MB/s', output))
    if not matches:
        raise ValueError(f"Could not find speed in output: {output}")
    return float(matches[-1].group(1))

def benchmark_file(compressed_file: Path, dictionary_path: Path = None) -> float:
    """Benchmark decompression of a file using zstd."""
    cmd = ['zstd', '-b', '-d', '-i0']
    if dictionary_path:
        cmd.extend(['-D', str(dictionary_path)])
    cmd.append(str(compressed_file))
    
    result = subprocess.run(cmd, capture_output=True, text=True, check=True)
    return extract_speed(result.stdout)