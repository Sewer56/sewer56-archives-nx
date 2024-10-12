import sys

def strip_whitespace(file_path):
    if file_path.endswith('.md'):
        with open(file_path, 'r') as file:
            lines = file.readlines()

        # Check if the last line ends with a newline
        has_newline_at_end = lines and lines[-1].endswith('\n')

        with open(file_path, 'w') as file:
            for i, line in enumerate(lines):
                # Remove trailing spaces
                line = line.rstrip()

                # Add newline to all lines except the last one if it didn't originally have a newline
                if i < len(lines) - 1 or has_newline_at_end:
                    line += '\n'

                file.write(line)

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: python strip_whitespace.py <file_path>")
        sys.exit(1)

    file_path = sys.argv[1]
    strip_whitespace(file_path)