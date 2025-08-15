from names_dataset import NameDataset

import unicodedata
import write_first_names

OUTPUT_PATH = write_first_names.OUTPUT_DIR + '/family_names.txt'

# Is this a char we are using? Just needs to be searhable via ASCII
# and I also get rid of spaces to avoid confusion here
def can_normalise_to_ascii(ch):
    if ch.isspace():
        return False
    if ord(ch) < 128:
        return True
    decomposed = unicodedata.normalize('NFD', ch)
    stripped = decomposed.encode('ascii', 'ignore').decode('ascii')
    return bool(stripped)

# The library takes time to initialize because the database is massive. A tip is to include its initialization in your app's startup process.
def main():
    nd = NameDataset()
    all_names = nd.last_names
    names = []

    # Remove names with "hard to type" characters or spaces etc
    for name in all_names.keys(): # type: ignore
        bad = False
        name = str(name)
        for ch in name:
            if not can_normalise_to_ascii(ch):
                bad = True
                break
        if bad:
            continue
        names.append(name)

    # Write the names file
    with open(OUTPUT_PATH, 'w') as f:
        f.truncate(0)
        for name in names:
            f.write(name)
            f.write('\n')

if __name__ == "__main__":
    main()
