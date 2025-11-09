import os
from git import Repo  # pip install gitpython

CITIES_REPO_URL = "https://github.com/giraugh/world-cities-dataset"
CITIES_REPO_PATH = "./input/world-cities/"
OUTPUT_DIR = "./output/"


def main():
    if not os.path.exists(CITIES_REPO_PATH):
        print("Cloning repo...")
        _ = Repo.clone_from(
            CITIES_REPO_URL,
            CITIES_REPO_PATH,
            depth=1,
        )

    # Just copy the file line by line
    # excluding any rows with more than one `,` in it
    # (I moved the processing into a fork of the dataset repo since it required running a CLI tool)
    in_path = f"{CITIES_REPO_PATH}cities.txt"
    out_path = f"{OUTPUT_DIR}cities.txt"
    with open(in_path, "r") as in_f:
        with open(out_path, "w") as out_f:
            out_f.truncate(0)
            for line in in_f:
                segs = line.split(",")
                if len(segs) == 2:
                    out_f.write(segs[0].strip() + ":" + segs[1].strip() + "\n")


if __name__ == "__main__":
    main()
