#!/usr/bin/python
import os
import sys
import subprocess
from pathlib import Path

def process_directory(input_dir, output_dir, transform_script, operation):
    """
    Process all DDS files in input directory and save transformed files to output directory.
    
    Args:
        input_dir (str): Source directory with DDS files
        output_dir (str): Destination directory for transformed files
        transform_script (str): Path to the transformation script
        operation (str): Either '-t' for transform or '-u' for untransform
    """
    # Create output directory if it doesn't exist
    Path(output_dir).mkdir(parents=True, exist_ok=True)
    
    # Counter for processed files
    processed_count = 0
    error_count = 0
    
    # Walk through all files in input directory
    for root, _, files in os.walk(input_dir):
        for file in files:
            if file.lower().endswith('.dds'):
                # Create input and output paths
                input_path = Path(root) / file
                # Maintain directory structure in output
                rel_path = Path(root).relative_to(input_dir)
                output_subdir = Path(output_dir) / rel_path
                output_subdir.mkdir(parents=True, exist_ok=True)
                output_path = output_subdir / file
                
                print(f"Processing: {input_path}")
                
                try:
                    # Run transformation script
                    subprocess.run([
                        sys.executable,
                        transform_script,
                        operation,
                        str(input_path),
                        str(output_path)
                    ], check=True, capture_output=True, text=True)
                    
                    processed_count += 1
                    print(f"Success: {output_path}")
                    
                except subprocess.CalledProcessError as e:
                    error_count += 1
                    print(f"Error processing {input_path}:")
                    print(f"Error message: {e.stderr}")
    
    print(f"\nProcessing complete:")
    print(f"Successfully processed: {processed_count} files")
    print(f"Errors encountered: {error_count} files")

def main():
    if len(sys.argv) != 5:
        print("Usage: python batch_transform.py <transform_script> <operation> <input_dir> <output_dir>")
        print("  operation: -t for transform or -u for untransform")
        print("Example:")
        print("  python batch_transform.py transform_script.py -t /path/to/input /path/to/output")
        sys.exit(1)
    
    transform_script = sys.argv[1]
    operation = sys.argv[2]
    input_dir = sys.argv[3]
    output_dir = sys.argv[4]
    
    # Validate operation
    if operation not in ['-t', '-u']:
        print("Error: operation must be either '-t' or '-u'")
        sys.exit(1)
    
    # Validate input directory
    if not os.path.exists(input_dir):
        print(f"Error: Input directory '{input_dir}' does not exist")
        sys.exit(1)
    
    # Validate transform script
    if not os.path.exists(transform_script):
        print(f"Error: Transform script '{transform_script}' does not exist")
        sys.exit(1)
    
    print(f"Input directory: {input_dir}")
    print(f"Output directory: {output_dir}")
    print(f"Operation: {'Transform' if operation == '-t' else 'Untransform'}")
    
    process_directory(input_dir, output_dir, transform_script, operation)

if __name__ == '__main__':
    main()