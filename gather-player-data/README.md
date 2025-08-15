# get popular names
A Little python script which pulls some popular baby names and reduces the data down into a more consumable small list of names.

In particular, it splits the names into three age classes:
 - `young`
 - `mature`
 - `old`

And it writes the names in a sort of popularity-order to a text file seperated by newlines.

I also included a `random_name.py` script to test and demonstrate how to get a random name without loading the whole file.

## Usage

(*requires `uv`*)

- **Write the first names** - `uv run src/write_first_names.py`
 - (Will clone the name repo if not already present)
- **Write the family names** - `uv run src/write_family_names.py`
 - (Pulls down a lot of data)
- **Get a random name as a test** - `uv run src/random_name.py`
