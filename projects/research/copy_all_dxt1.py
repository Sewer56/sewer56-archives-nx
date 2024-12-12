#!/usr/bin/python
import os
import shutil
import argparse

def check_dxt1_in_file(file_path):
    """Check if 'DXT1' exists in first 100 bytes of the file."""
    try:
        with open(file_path, 'rb') as f:
            # Read first 100 bytes
            header = f.read(100)
            return b'DXT1' in header
    except Exception as e:
        print(f"Error reading file {file_path}: {e}")
        return False

def copy_dxt1_files(source_dir, dest_dir):
    """
    Find files containing 'DXT1' in first 100 bytes and copy them to destination.
    
    Args:
        source_dir (str): Source directory path
        dest_dir (str): Destination directory path
    """
    # Create destination directory if it doesn't exist
    if not os.path.exists(dest_dir):
        os.makedirs(dest_dir)
    
    # Counter for found files
    found_count = 0
    
    # Walk through all files in source directory
    for root, _, files in os.walk(source_dir):
        for file in files:
            source_path = os.path.join(root, file)
            
            # Check if file contains DXT1 in first 100 bytes
            if check_dxt1_in_file(source_path):
                # Create relative path to maintain directory structure
                rel_path = os.path.relpath(root, source_dir)
                dest_path = os.path.join(dest_dir, rel_path)
                
                # Create subdirectories if they don't exist
                if not os.path.exists(dest_path):
                    os.makedirs(dest_path)
                
                # Copy file to destination
                dest_file = os.path.join(dest_path, file)
                shutil.copy2(source_path, dest_file)
                
                print(f"Copied: {source_path} -> {dest_file}")
                found_count += 1
    
    print(f"\nTotal files copied: {found_count}")

def main():
    # Set up command line arguments
    parser = argparse.ArgumentParser(description='Copy files containing DXT1 in first 100 bytes')
    parser.add_argument('source', help='Source directory path')
    parser.add_argument('destination', help='Destination directory path')
    
    args = parser.parse_args()
    
    # Convert to absolute paths
    source_dir = os.path.abspath(args.source)
    dest_dir = os.path.abspath(args.destination)
    
    # Check if source directory exists
    if not os.path.exists(source_dir):
        print(f"Error: Source directory '{source_dir}' does not exist")
        return
    
    print(f"Searching in: {source_dir}")
    print(f"Copying to: {dest_dir}")
    
    copy_dxt1_files(source_dir, dest_dir)

if __name__ == "__main__":
    main()
