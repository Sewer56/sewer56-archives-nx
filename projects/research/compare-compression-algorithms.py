#!/usr/bin/env python

import os
import sys
import subprocess
import time
from datetime import datetime
from pathlib import Path
from typing import Dict, Tuple, List
import shutil

def get_directory_size(path: str) -> int:
    """Calculate total size of a directory in bytes."""
    total = 0
    for dirpath, _, filenames in os.walk(path):
        for f in filenames:
            fp = os.path.join(dirpath, f)
            total += os.path.getsize(fp)
    return total

def format_size(size_bytes: int) -> str:
    """Convert bytes to human readable format."""
    for unit in ['B', 'KiB', 'MiB', 'GiB', 'TiB']:
        if size_bytes < 1024.0:
            return f"{size_bytes:.2f} {unit}"
        size_bytes /= 1024.0
    return f"{size_bytes:.2f} PiB"

def get_files_list(directory: str) -> List[str]:
    """Get list of all files in directory and subdirectories."""
    files = []
    for dirpath, _, filenames in os.walk(directory):
        for filename in filenames:
            files.append(os.path.join(dirpath, filename))
    return files

def create_output_structure(output_base: str, input_path: str, file_path: str, compressor: str) -> str:
    """Create and return the output directory structure for a file."""
    rel_path = os.path.relpath(os.path.dirname(file_path), input_path)
    output_dir = os.path.join(output_base, compressor, rel_path)
    os.makedirs(output_dir, exist_ok=True)
    return output_dir

def compress_file(file_path: str, output_dir: str, compressor: str, input_base_path: str) -> Tuple[int, float]:
    """Compress a single file using specified compressor."""
    rel_path = os.path.relpath(file_path, input_base_path)
    output_path = os.path.join(output_dir, rel_path)
    start_time = time.time()
    
    try:
        if compressor == '7z':
            output_file = f"{output_path}.7z.xz"
            cmd = ['7z', 'a', '-txz', '-mx=9', output_file, file_path]
        
        elif compressor == 'xz':
            output_file = f"{output_path}.xz"
            # xz adds .xz extension automatically like bzip3
            cmd = ['xz', '-k', '-e', '-z', '-9', '--force', file_path]
            # Path to the file xz will create
            input_xz = f"{file_path}.xz"
        
        elif compressor == 'bzip3':
            output_file = f"{output_path}.bz3"
            cmd = ['bzip3', '-k', '-j', '12', '-e', file_path, '-f']
            # Path to the file bzip3 will create
            input_bz3 = f"{file_path}.bz3"
        
        elif compressor == 'kanzi 7':
            output_file = f"{output_path}.knz"
            cmd = ['kanzi', '-c', f'--input={file_path}', f'--output={output_file}', '-l', '7', '--force']

        elif compressor == 'kanzi 6':
            output_file = f"{output_path}.knz"
            cmd = ['kanzi', '-c', f'--input={file_path}', f'--output={output_file}', '-l', '6', '--force']

        elif compressor == 'kanzi 5':
            output_file = f"{output_path}.knz"
            cmd = ['kanzi', '-c', f'--input={file_path}', f'--output={output_file}', '-l', '5', '--force']

        result = subprocess.run(cmd, capture_output=True, text=True)
        if result.returncode != 0:
            print(f"Error compressing {file_path} with {compressor}:")
            print(result.stderr)
            return 0, 0

        # Special handling for bzip3 and xz
        if compressor == 'bzip3':
            if os.path.exists(input_bz3):
                os.makedirs(os.path.dirname(output_file), exist_ok=True)
                shutil.move(input_bz3, output_file)
            else:
                print(f"Error: bzip3 compressed file not found: {input_bz3}")
                return 0, 0
        elif compressor == 'xz':
            if os.path.exists(input_xz):
                os.makedirs(os.path.dirname(output_file), exist_ok=True)
                shutil.move(input_xz, output_file)
            else:
                print(f"Error: xz compressed file not found: {input_xz}")
                return 0, 0
        
        compressed_size = os.path.getsize(output_file)
        return compressed_size, time.time() - start_time
    
    except Exception as e:
        print(f"Error processing {file_path} with {compressor}: {str(e)}")
        return 0, 0

def main():
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} <input_directory>")
        sys.exit(1)

    input_dir = sys.argv[1]
    if not os.path.isdir(input_dir):
        print(f"Error: {input_dir} is not a directory")
        sys.exit(1)

    # Get the parent directory of the input directory
    parent_dir = os.path.dirname(os.path.abspath(input_dir))
    input_name = os.path.basename(input_dir)
    
    # Create base output directory with timestamp beside the input directory
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    base_output_dir = os.path.join(parent_dir, f"{input_name}_compressed_{timestamp}")
    compressors = ['7z', 'xz', 'bzip3', 'kanzi 7']

    # Get list of all files
    files = get_files_list(input_dir)
    if not files:
        print("No files found in input directory")
        sys.exit(1)

    # Get original total size
    original_size = get_directory_size(input_dir)
    print(f"Original size: {format_size(original_size)}")
    print(f"Total files to process: {len(files)}")
    print("Starting compression benchmark...\n")

    # Process each compressor
    results: Dict[str, Dict[str, float]] = {}
    total_start_time = time.time()

    for comp in compressors:
        print(f"\nRunning {comp} compression...")
        total_compressed_size = 0
        total_comp_time = 0
        files_processed = 0
        
        for file_path in files:
            output_dir = create_output_structure(base_output_dir, input_dir, file_path, comp)
            size, duration = compress_file(file_path, output_dir, comp, input_dir)
            
            if size > 0:
                total_compressed_size += size
                total_comp_time += duration
                files_processed += 1
                
                # Show progress
                print(f"\rProcessed: {files_processed}/{len(files)} files", end="", flush=True)
        
        print()  # New line after progress
        
        if files_processed > 0:
            results[comp] = {
                'size': total_compressed_size,
                'ratio': (total_compressed_size / original_size) * 100,
                'time': total_comp_time,
                'speed': original_size / (1024 * 1024 * total_comp_time) if total_comp_time > 0 else 0
            }

    total_time = time.time() - total_start_time

    # Print results
    print("\n=== Compression Benchmark Results ===")
    print(f"Input directory: {input_dir}")
    print(f"Original size: {format_size(original_size)}")
    print(f"Files processed: {len(files)}")
    print("\nCompressor Results:")
    print(f"{'Tool':<10} {'Size':<15} {'Ratio':<15} {'Time':<10} {'Speed':<15}")
    print("-" * 65)

    for comp in compressors:
        if comp in results:
            r = results[comp]
            print(f"{comp:<10} "
                  f"{format_size(r['size']):<15} "
                  f"{r['ratio']:.2f}%{' ':<11} "
                  f"{r['time']:.1f}s{' ':<6} "
                  f"{r['speed']:.2f} MB/s")

    print(f"\nTotal benchmark time: {total_time:.1f}s")
    print(f"Results stored in: {base_output_dir}")

if __name__ == "__main__":
    main()