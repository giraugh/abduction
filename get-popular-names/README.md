# get popular names
A Little python script which pulls some popular baby names and reduces the data down into a more consumable small list of names.

In particular, it splits the names into three age classes:
 - `young`
 - `mature`
 - `old`

And it writes the names in a sort of popularity-order to a text file seperated by newlines. It also outputs an index file (the adjacent .idx) file with byte offsets for each name stored as 16-bit uints. This way you can choose a random byte pair in the index, lookup that offset and then read until a newline in order to get a random name without reading either file into memory.
