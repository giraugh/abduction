import os
import sys
import random

import write_names

def main(age_class):
    # valid age class?
    if age_class not in write_names.AGE_CLASSES.keys():
        raise ValueError(f'Invalid age class "{age_class}"')

    idx_path = f'{write_names.OUTPUT_DIR}/{age_class}.idx'
    names_path = f'{write_names.OUTPUT_DIR}/{age_class}.txt'

    # Choose a random pair of bytes from within the index
    name_count = os.path.getsize(idx_path) // 2 # stored as 2 bytes so we half
    name_index = random.randrange(0, name_count - 1, 1)

    # Load the index file and determine the byte offset
    # for the chosen name
    with open(idx_path, 'rb') as f:
        f.seek(name_index * 2)
        offset_b = f.read(2)
        offset = int.from_bytes(offset_b, 'big')

    # Now go read from that offset till we hit a newline
    with open(names_path, 'r') as f:
        name = ""
        f.seek(offset)
        while True:
            c = f.read(1)
            if c == '\n' or c == '':
                break
            name += c
    print(name)

if __name__ == "__main__":
    if len(sys.argv) < 2:
        raise ValueError('Expected age class argument')
    main(sys.argv[1])
