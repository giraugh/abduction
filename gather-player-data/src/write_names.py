import glob
import os
import struct
import math
import pandas as pd
from git import Repo  # pip install gitpython

NAME_REPO_URL = "https://github.com/aruljohn/popular-baby-names"
NAMES_REPO_PATH = "./input/popular-baby-names/"
OUTPUT_DIR = './output/'

# The last year for each age range (if higher than all, discard)
# (order is important)
AGE_CLASSES = {
    "old": 1900,
    "mature": 1960,
    "young": 1990,
}

def main():
    if not os.path.exists(NAMES_REPO_PATH):
        print("Cloning repo...")
        Repo.clone_from(NAME_REPO_URL, NAMES_REPO_PATH)

    # Initialise data frame
    df = pd.DataFrame()

    # Okay, the goal is to take all the years in the provided classes and find the unique set of names in each
    # or maybe with weights for popularity?
    for year_dir in glob.glob(f'{NAMES_REPO_PATH}/*/'):
        # Determine year
        year = os.path.basename(year_dir[:-1])
        year = int(year)

        # Which class?
        age_class = None
        for ac, first_year in AGE_CLASSES.items():
            if year >= first_year:
                age_class = ac
        if age_class == None:
            continue

        # Load the csv
        with open(f'{year_dir}/girl_boy_names_{year}.csv', 'r') as csv_f:
            # Load the csv
            year_names_data = pd.read_csv(csv_f)

            # Add in year and class
            year_names_data['Age Class'] = age_class

            # Add to frame
            df = pd.concat([df, year_names_data]).reset_index(drop=True)

    # Now we want to unpivot on gender (coz who cares about something like that)
    df = df.melt(
        id_vars=['Rank', 'Age Class'],
        value_vars=['Girl Name', 'Boy Name'],
        var_name='Gender',
        value_name='name'
    ).drop(columns=['Gender'])

    # Snake case all the columns
    df.columns = df.columns.str.strip().str.lower().str.replace(' ', '_')

    # Now I want to segregate the different age classes into different frames
    # and write them to csvs
    age_dfs = { ac: df[df['age_class'] == ac] for ac in AGE_CLASSES.keys()}

    # And for each, remove the age class column (as its obvious from the context)
    for ac in age_dfs.keys():
        age_dfs[ac] = age_dfs[ac]\
            .drop(columns=['age_class'])\
            .groupby('name', as_index=False)\
            .agg(
                count=('name', 'size'),
                rank=('rank', 'max')
            )\
            .rename({ 'size': 'count' })
        age_dfs[ac].sort_values(by='count', ascending=False, inplace=True) # type: ignore

    # How many did we get for each?
    for ac in age_dfs.keys():
        print(ac, len(age_dfs[ac]))

    # For the best performance n stuff, we're gonna do this a slightly wacky way
    # rather than use like csv or something - instead we're gonna do this:
    #  - store each name on a new line of a .txt file
    #  - generate an index with the byte offset of each line as a 16-bit uint.
    #  - This way, we can seek to a random index, read it, then go to that
    #    byte in the name file and read until a newline.
    print('Writing name files')
    for ac in age_dfs.keys():
        # Concatenate the names seperated by newlines
        # and keep track of the byte offset for each
        offset = 0
        names = ''
        indexes = []
        for name in age_dfs[ac]['name']:
            # Record offset
            assert offset < math.pow(2, 16)
            indexes.append(struct.pack('>H', offset))

            # Add name and increment offset
            names += f'{name}\n'
            offset += len(name) + 1

        # Write the names text file
        with open(f'{OUTPUT_DIR}/{ac}.txt', 'w') as f:
            f.truncate(0)
            f.write(names)

        # Write the index
        with open(f'{OUTPUT_DIR}/{ac}.idx', 'wb') as f:
            f.truncate(0)
            for index in indexes:
                f.write(index)



if __name__ == "__main__":
    main()
