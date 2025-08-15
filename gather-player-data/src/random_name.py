import os
import sys
import random

def main(file_path):
    # Choose a random start byte
    file_size = os.path.getsize(file_path)
    start_off = random.randrange(0, file_size, 1)

    # Load the file at that index and seek backwards till we hit a newline
    with open(file_path, 'rb') as f:
        # Go back to previous NL
        if start_off != 0:
            pos = start_off - 1
            while pos >= 0:
                f.seek(pos)
                byte = f.read(1)
                if byte == b'\n':
                    break
                pos -= 1

        # Now read until a NL
        bytes = []
        while True:
            c = f.read(1)
            if c == '' or c == b'\n':
                break
            elif c:
                bytes.append(c)
        bytes_ = bytearray(b''.join(bytes))
        decoded = bytes_.decode('utf-8')
        print(decoded)

if __name__ == "__main__":
    if len(sys.argv) < 2:
        raise ValueError('Expected path argument')
    main(sys.argv[1])
