import glob
import os
import pandas as pd
from git import Repo  # pip install gitpython

CITIES_REPO_URL = "https://github.com/datasets/world-cities"
CITIES_REPO_PATH = "./input/world-cities/"
OUTPUT_DIR = './output/'

def main():
    if not os.path.exists(CITIES_REPO_PATH):
        print("Cloning repo...")
        Repo.clone_from(CITIES_REPO_URL, CITIES_REPO_PATH)

    # Initialise data frame
    df = pd.DataFrame()

    # Load the csv
    with open(f'{CITIES_REPO_PATH}/data/world-cities.csv', 'r') as csv_f:
        # Load the csv
        cities_data = pd.read_csv(csv_f)

        # Add to frame
        df = pd.concat([df, cities_data]).reset_index(drop=True)

    # We want entries like "City, Country" (or perhaps Subcountry)
    # Write em in with new line seperators
    print('Writing cities')
    with open(f'{OUTPUT_DIR}/cities.txt', 'w') as f:
        f.truncate(0)
        for i, row in df.iterrows():
            # Skip if too long or contains special characters
            line = f"{row['name']}, {row['country']}\n"

            if "/" in line or "(" in line or "," in row['name'] or "," in row['country'] or len(line) > 30:
                continue

            # Write the city
            f.write(line)

if __name__ == "__main__":
    main()
