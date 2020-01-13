"""
Simple script to generate random json files, used in conjuction with current crate
to test the performance capabilities of i/o.
"""

import json
from faker import Faker
import random
from random import randint
import sys
fake = Faker('en_US')


def gen(n):
    json_f = dict()
    for i in range(int(n)):
        my_dict = {
            "id": i,
            'name': fake.name(),
            'email': fake.email(),
            'hp': randint(0, 10000),
            'mana': randint(0, 10000),
            'vit': randint(0, 10000)
        }
        json_f[i] = my_dict

    return json.dumps(json_f, indent=4)


if __name__ == "__main__":
    args = sys.argv
    print(args)
    fname = args[1]
    n = args[2]

    with open(fname, "w") as f:
        j = gen(n)
        f.writelines(j)
