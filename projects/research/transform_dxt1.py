#!/usr/bin/python
import sys
import struct
from pathlib import Path

def read_dds_header(f):
    """Read DDS header and return size info and data offset"""
    # Skip magic number
    f.seek(4)
    # Read header size
    header_size = struct.unpack('<I', f.read(4))[0]
    # Read height and width
    f.seek(12)
    height = struct.unpack('<I', f.read(4))[0]
    width = struct.unpack('<I', f.read(4))[0]
    return width, height, header_size + 4

def transform_dxt1(input_path, output_path):
    """Transform DXT1 texture by separating endpoints and indices"""
    with open(input_path, 'rb') as f:
        # Read header
        width, height, data_offset = read_dds_header(f)
        
        # Save header
        f.seek(0)
        header = f.read(data_offset)
        
        # Read blocks
        blocks = []
        while True:
            block = f.read(8)  # DXT1 block is 8 bytes
            if not block or len(block) < 8:
                break
            blocks.append(block)

    # Separate endpoints and indices
    endpoints = bytearray()
    indices = bytearray()
    
    for block in blocks:
        # First 4 bytes are endpoints
        endpoints.extend(block[0:4])
        # Last 4 bytes are indices
        indices.extend(block[4:8])

    # Write transformed file
    with open(output_path, 'wb') as f:
        # Write header
        f.write(header)
        # Write all endpoints followed by all indices
        f.write(endpoints)
        f.write(indices)

def untransform_dxt1(input_path, output_path):
    """Restore original DXT1 format from transformed texture"""
    with open(input_path, 'rb') as f:
        # Read header
        width, height, data_offset = read_dds_header(f)
        
        # Save header
        f.seek(0)
        header = f.read(data_offset)
        
        # Calculate number of blocks
        block_count = ((width + 3) // 4) * ((height + 3) // 4)
        
        # Read separated data
        f.seek(data_offset)
        endpoints = f.read(block_count * 4)
        indices = f.read(block_count * 4)

    # Write untransformed file
    with open(output_path, 'wb') as f:
        # Write header
        f.write(header)
        # Recombine blocks
        for i in range(block_count):
            f.write(endpoints[i*4:(i+1)*4])
            f.write(indices[i*4:(i+1)*4])

if __name__ == '__main__':
    if len(sys.argv) != 4 or sys.argv[1] not in ['-t', '-u']:
        print("Usage:")
        print("  Transform:   python script.py -t input.dds output.dds")
        print("  Untransform: python script.py -u input.dds output.dds")
        sys.exit(1)

    operation = sys.argv[1]
    input_path = sys.argv[2]
    output_path = sys.argv[3]

    if operation == '-t':
        transform_dxt1(input_path, output_path)
    else:
        untransform_dxt1(input_path, output_path)