#!/usr/bin/env python3

import os
import sys
import argparse
from pathlib import Path

def get_file_size(path):
    """Get file size in bytes"""
    return os.path.getsize(path) if os.path.exists(path) else 0

def format_size(size):
    """Format size in bytes to human readable format using binary prefixes"""
    for unit in ['B', 'KiB', 'MiB', 'GiB']:
        if size < 1024:
            return f"{size:.2f}{unit}"
        size /= 1024
    return f"{size:.2f}TiB"

def get_percent_diff(larger, smaller):
    """Calculate percentage difference"""
    return ((larger - smaller) / larger) * 100

def get_base_name(filepath):
    """Get the base name of a file (path up to first dot in filename)"""
    dirpath, filename = os.path.split(filepath)
    base = filename.split('.')[0]
    return os.path.join(dirpath, base)

def get_files_recursive(directory):
    """Get all files recursively with full paths, keyed by base name"""
    files = {}
    for root, _, filenames in os.walk(directory):
        for filename in filenames:
            full_path = os.path.join(root, filename)
            base_path = get_base_name(full_path)
            files[base_path] = full_path
    return files

def main():
    parser = argparse.ArgumentParser(description='Compare file sizes across multiple directories')
    parser.add_argument('dirs', nargs='+', help='Directories to compare')
    args = parser.parse_args()

    # Build folders dictionary and collect files
    folders = {}
    
    for dir_path in args.dirs:
        if not os.path.exists(dir_path):
            print(f"Error: Directory '{dir_path}' does not exist", file=sys.stderr)
            sys.exit(1)
        name = os.path.basename(dir_path)
        files = get_files_recursive(dir_path)
        folders[name] = {
            "path": dir_path,
            "files": files,
            "total_size": sum(os.path.getsize(f) for f in files.values())
        }

    # Create sets of base paths for comparison
    all_base_paths = set()
    for folder_info in folders.values():
        for base_path in folder_info["files"].keys():
            # Get the path relative to the folder
            rel_path = os.path.relpath(base_path, folder_info["path"])
            all_base_paths.add(rel_path)

    # Calculate column widths
    headers = ["Path"] + list(folders.keys()) + ["Best", "Savings %"]
    col_widths = [len(h) for h in headers]
    rows = []
    
    best_combination_total = 0

    for rel_base_path in sorted(all_base_paths):
        sizes = {}
        full_paths = {}  # Store the actual paths for display
        for folder_name, folder_info in folders.items():
            base_path = os.path.join(folder_info["path"], rel_base_path)
            if base_path in folder_info["files"]:
                actual_path = folder_info["files"][base_path]
                size = get_file_size(actual_path)
                sizes[folder_name] = size
                full_paths[folder_name] = actual_path

        valid_sizes = {k: v for k, v in sizes.items() if v > 0}
        if valid_sizes:
            best_method = min(valid_sizes.items(), key=lambda x: x[1])
            worst_method = max(valid_sizes.items(), key=lambda x: x[1])
            savings = get_percent_diff(worst_method[1], best_method[1])
            best_combination_total += best_method[1]
            
            # Use the relative path without extensions
            display_path = rel_base_path
            row = [display_path]
            for folder_name in folders.keys():
                size = sizes.get(folder_name, 0)
                row.append("N/A" if size == 0 else format_size(size))
            row.append(best_method[0])
            row.append(f"{savings:.2f}%")
            
            # Update column widths
            for i, item in enumerate(row):
                col_widths[i] = max(col_widths[i], len(str(item)))
            rows.append(row)

    # Print table header
    print("\nFile Size Comparisons:")
    print("=" * sum(col_widths + [len(headers) - 1]))
    header_format = " ".join(f"{h:<{w}}" for h, w in zip(headers, col_widths))
    print(header_format)
    print("-" * sum(col_widths + [len(headers) - 1]))

    # Print table rows
    for row in rows:
        row_format = " ".join(f"{item:<{w}}" for item, w in zip(row, col_widths))
        print(row_format)

    print("=" * sum(col_widths + [len(headers) - 1]))

    # Print totals
    print("\nFolder Totals:")
    for folder_name, folder_info in folders.items():
        print(f"{folder_name}: {format_size(folder_info['total_size'])}")

    print(f"\nBest Combination Total: {format_size(best_combination_total)}")
    max_total = max(folder_info["total_size"] for folder_info in folders.values())
    print(f"Potential Space Saving vs Worst: {format_size(max_total - best_combination_total)}")

if __name__ == "__main__":
    main()