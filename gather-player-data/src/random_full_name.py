
import random
import write_family_names
import write_first_names
import random_name

def main():
    # Random age class
    ac = random.choice(list(write_first_names.AGE_CLASSES.keys()))

    # Get the names
    first_name = random_name.get_random_name(f'{write_first_names.OUTPUT_DIR}/{ac}.txt')
    last_name = random_name.get_random_name(write_family_names.OUTPUT_PATH)

    print(f'{first_name} {last_name}')

if __name__ == "__main__":
    main()
